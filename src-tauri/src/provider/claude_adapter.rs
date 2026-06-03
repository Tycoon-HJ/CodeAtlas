use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter};

use super::ProviderAdapter;
use crate::log_info;
use crate::log_error;
use crate::log_debug;
use crate::log_warn;
use crate::models::{ClaudeOutput, ToolCall};
use crate::pty;

/// Global state for child processes (used by -p mode), so we can kill them if user aborts.
type ChildProcesses = Mutex<HashMap<String, Child>>;
static CHILD_PROCESSES: once_cell::sync::Lazy<ChildProcesses> =
    once_cell::sync::Lazy::new(|| Mutex::new(HashMap::new()));

fn get_child_processes() -> &'static ChildProcesses {
    &CHILD_PROCESSES
}

/// Maps app session_id -> Claude CLI session_id for session continuity.
type ClaudeSessionMap = Mutex<HashMap<String, String>>;
static CLAUDE_SESSIONS: once_cell::sync::Lazy<ClaudeSessionMap> =
    once_cell::sync::Lazy::new(|| Mutex::new(HashMap::new()));

/// Tracks which sessions are using PTY mode.
type PtySessionSet = Mutex<std::collections::HashSet<String>>;
static PTY_SESSIONS: once_cell::sync::Lazy<PtySessionSet> =
    once_cell::sync::Lazy::new(|| Mutex::new(std::collections::HashSet::new()));

/// Tracks trust prompt resolution per session.
type PtyTrustMap = Mutex<HashMap<String, std::sync::Arc<std::sync::Mutex<bool>>>>;
pub static PTY_TRUST_RESOLVED: once_cell::sync::Lazy<PtyTrustMap> =
    once_cell::sync::Lazy::new(|| Mutex::new(HashMap::new()));

/// Kill a running Claude process for a session.
pub fn kill_session_process(session_id: &str) {
    // Kill PTY session if exists
    if let Ok(mut set) = PTY_SESSIONS.lock() {
        if set.remove(session_id) {
            pty::get_pty_manager().close(session_id);
            log_info!("claude", "Killed PTY session {}", session_id);
        }
    }
    // Kill -p mode process if exists
    if let Ok(mut processes) = get_child_processes().lock() {
        if let Some(mut child) = processes.remove(session_id) {
            let _ = child.kill();
            log_info!("claude", "Killed -p process for session {}", session_id);
        }
    }
    // Cleanup trust resolved entry
    if let Ok(mut map) = PTY_TRUST_RESOLVED.lock() {
        map.remove(session_id);
    }
    clear_claude_session(session_id);
}

/// Clear stored Claude session ID for an app session.
pub fn clear_claude_session(session_id: &str) {
    if let Ok(mut map) = CLAUDE_SESSIONS.lock() {
        map.remove(session_id);
    }
}

pub struct ClaudeAdapter;

impl ClaudeAdapter {
    fn build_default_cmd(home: &str, base_args: Vec<String>) -> (String, Vec<String>) {
        if cfg!(target_os = "windows") {
            let npm_claude = format!("{}\\AppData\\Roaming\\npm\\claude.cmd", home);
            if std::path::Path::new(&npm_claude).exists() {
                let mut a = vec!["/c".to_string(), npm_claude];
                a.extend(base_args);
                ("cmd".to_string(), a)
            } else {
                let mut a = vec!["/c".to_string(), "claude".to_string()];
                a.extend(base_args);
                ("cmd".to_string(), a)
            }
        } else {
            ("claude".to_string(), base_args)
        }
    }

    /// Check if a message is a slash command (e.g., /help, /compact).
    fn is_slash_command(message: &str) -> bool {
        message.trim().starts_with('/')
    }

    /// Start a PTY session for interactive Claude CLI.
    fn start_pty_session(
        app: &AppHandle,
        session_id: &str,
        message: &str,
        working_dir: &str,
        claude_path: Option<&str>,
        claude_config_path: Option<&str>,
    ) -> Result<(), String> {
        let session_id_clone = session_id.to_string();
        let message = message.to_string();
        let working_dir = working_dir.to_string();
        let claude_path_str = claude_path.map(|s| s.to_string());
        let claude_config_str = claude_config_path.map(|s| s.to_string());
        let app = app.clone();

        // Check for resume session
        let resume_id = {
            if let Ok(map) = CLAUDE_SESSIONS.lock() {
                map.get(session_id).cloned()
            } else {
                None
            }
        };

        let mut args = vec![
            "--output-format".to_string(), "stream-json".to_string(),
            "--verbose".to_string(),
            "--permission-mode".to_string(), "default".to_string(),
        ];

        if let Some(ref rid) = resume_id {
            args.push("--resume".to_string());
            args.push(rid.clone());
            log_info!("claude", "[{}] PTY resuming Claude session: {}", session_id, rid);
        }

        if let Some(ref config) = claude_config_str {
            if !config.is_empty() {
                args.push("--config-file".to_string());
                args.push(config.clone());
            }
        }

        // Mark as PTY session
        if let Ok(mut set) = PTY_SESSIONS.lock() {
            set.insert(session_id_clone.clone());
        }

        let path_ref = claude_path_str.as_deref();

        // Clone values for the PTY output handler closure
        let sid_for_handler = session_id_clone.clone();
        let app_for_handler = app.clone();
        let sid_for_writer = session_id_clone.clone();
        let app_for_writer = app.clone();
        let msg_for_writer = message.clone();

        // Buffer for accumulating non-JSON output to detect multi-line prompts
        let non_json_buffer: std::sync::Arc<std::sync::Mutex<String>> = std::sync::Arc::new(std::sync::Mutex::new(String::new()));
        let trust_prompted: std::sync::Arc<std::sync::Mutex<bool>> = std::sync::Arc::new(std::sync::Mutex::new(false));
        let trust_resolved: std::sync::Arc<std::sync::Mutex<bool>> = std::sync::Arc::new(std::sync::Mutex::new(false));

        // Store trust_resolved globally so respond_trust_prompt can set it
        if let Ok(mut map) = PTY_TRUST_RESOLVED.lock() {
            map.insert(session_id_clone.clone(), trust_resolved.clone());
        }

        // Spawn PTY
        pty::get_pty_manager().spawn(
            session_id,
            path_ref,
            &working_dir,
            args,
            move |line| {
                let session_id_clone = sid_for_handler.clone();
                let app = app_for_handler.clone();
                // Parse each line of output
                let parsed: serde_json::Value = match serde_json::from_str(&line) {
                    Ok(v) => v,
                    Err(_) => {
                        // Not JSON — accumulate and check for trust prompt
                        let truncated = if line.chars().count() > 100 { line.chars().take(100).collect::<String>() } else { line.clone() };
                        log_debug!("claude", "[{}] PTY non-JSON: {}", session_id_clone, truncated);

                        if let Ok(mut buf) = non_json_buffer.lock() {
                            buf.push_str(&line);
                            buf.push('\n');

                            // Check if we already prompted
                            let already_prompted = trust_prompted.lock().map(|v| *v).unwrap_or(true);
                            if !already_prompted {
                                let buf_lower = buf.to_lowercase();
                                let has_trust = buf_lower.contains("trust this folder")
                                    || buf_lower.contains("is this a project you created")
                                    || buf_lower.contains("yes, i trust")
                                    || (buf_lower.contains("trust") && buf_lower.contains("exit"))
                                    || (buf_lower.contains("security guide") && buf_lower.contains("trust"))
                                    || (buf_lower.contains("read, edit, and execute") && buf_lower.contains("trust"));
                                if has_trust {
                                    log_info!("claude", "[{}] Trust prompt detected in buffer ({} chars)", session_id_clone, buf.len());
                                    if let Ok(mut tp) = trust_prompted.lock() {
                                        *tp = true;
                                    }
                                    let _ = app.emit("claude-trust-prompt", serde_json::json!({
                                        "session_id": session_id_clone.clone(),
                                        "message": "Claude Code 需要您确认是否信任此工作目录。\n\n允许后 Claude 将可以读取、编辑和执行此目录中的文件。",
                                    }));
                                }
                            }
                        }
                        return;
                    }
                };

                let msg_type = parsed.get("type").and_then(|t| t.as_str()).unwrap_or("");

                match msg_type {
                    "assistant" => {
                        if let Some(message) = parsed.get("message") {
                            if let Some(content_arr) = message.get("content").and_then(|c| c.as_array()) {
                                let mut text_buf = String::new();
                                let mut thinking_buf = String::new();
                                let mut tool_calls_buf = Vec::new();

                                for block in content_arr {
                                    let block_type = block.get("type").and_then(|t| t.as_str()).unwrap_or("");
                                    match block_type {
                                        "text" => {
                                            if let Some(text) = block.get("text").and_then(|t| t.as_str()) {
                                                text_buf.push_str(text);
                                            }
                                        }
                                        "thinking" => {
                                            if let Some(thinking) = block.get("thinking").and_then(|t| t.as_str()) {
                                                thinking_buf.push_str(thinking);
                                            }
                                        }
                                        "tool_use" => {
                                            let tool_id = block.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                            let tool_name = block.get("name").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
                                            let tool_input = block.get("input").cloned().unwrap_or(serde_json::Value::Null);
                                            tool_calls_buf.push(ToolCall {
                                                tool_id: tool_id.clone(),
                                                tool_name: tool_name.clone(),
                                                tool_input: tool_input.clone(),
                                            });
                                            let _ = app.emit("claude-tool-call", serde_json::json!({
                                                "session_id": session_id_clone.clone(),
                                                "tool_id": tool_id,
                                                "tool_name": tool_name,
                                                "tool_input": tool_input,
                                                "tool_number": 0,
                                            }));
                                        }
                                        _ => {}
                                    }
                                }

                                if !text_buf.is_empty() || !thinking_buf.is_empty() {
                                    let _ = app.emit("claude-output", ClaudeOutput {
                                        session_id: session_id_clone.clone(),
                                        content: text_buf,
                                        thinking: thinking_buf,
                                        done: false,
                                        tool_calls: if tool_calls_buf.is_empty() { None } else { Some(tool_calls_buf) },
                                    });
                                }
                            }
                        }
                    }
                    "result" => {
                        let final_text = parsed.get("result").and_then(|r| r.as_str()).unwrap_or("");
                        let _ = app.emit("claude-output", ClaudeOutput {
                            session_id: session_id_clone.clone(),
                            content: final_text.to_string(),
                            thinking: String::new(),
                            done: true,
                            tool_calls: None,
                        });
                    }
                    "system" => {
                        if let Some(subtype) = parsed.get("subtype").and_then(|s| s.as_str()) {
                            log_info!("claude", "[{}] PTY system event: {}", session_id_clone, subtype);
                            if subtype == "init" {
                                let sid = parsed.get("session_id").and_then(|s| s.as_str())
                                    .or_else(|| parsed.get("data").and_then(|d| d.get("session_id")).and_then(|s| s.as_str()));
                                if let Some(sid) = sid {
                                    log_info!("claude", "[{}] PTY Claude session captured: {}", session_id_clone, sid);
                                    if let Ok(mut map) = CLAUDE_SESSIONS.lock() {
                                        map.insert(session_id_clone.clone(), sid.to_string());
                                    }
                                }
                            }
                        }
                    }
                    "permission" => {
                        // Permission request from Claude
                        log_info!("claude", "[{}] PTY permission request: {:?}", session_id_clone, parsed);
                        let _ = app.emit("claude-permission", serde_json::json!({
                            "session_id": session_id_clone.clone(),
                            "request": parsed,
                        }));
                    }
                    _ => {
                        log_debug!("claude", "[{}] PTY unknown type: {}", session_id_clone, msg_type);
                    }
                }
            },
        )?;

        // Send the first message via stdin
        let manager = pty::get_pty_manager();
        std::thread::spawn(move || {
            let sid = sid_for_writer.clone();
            // Wait for trust prompt to be resolved (if any), up to 5 minutes
            for _ in 0..3000 {
                std::thread::sleep(std::time::Duration::from_millis(100));
                if let Ok(map) = PTY_TRUST_RESOLVED.lock() {
                    if let Some(resolved) = map.get(&sid) {
                        if let Ok(r) = resolved.lock() {
                            if *r { break; }
                        }
                    } else {
                        // No trust prompt for this session, proceed immediately
                        break;
                    }
                }
            }
            // Small delay after trust resolution to let CLI initialize
            std::thread::sleep(std::time::Duration::from_millis(300));

            // Check if the PTY session is still alive before writing
            if !manager.has_session(&sid) {
                log_warn!("claude", "[{}] PTY session no longer exists, skipping write", sid);
                let _ = app_for_writer.emit("claude-output", ClaudeOutput {
                    session_id: sid.clone(),
                    content: "会话已终止".to_string(),
                    thinking: String::new(),
                    done: true,
                    tool_calls: None,
                });
                if let Ok(mut map) = PTY_TRUST_RESOLVED.lock() {
                    map.remove(&sid);
                }
                return;
            }

            log_info!("claude", "[{}] Sending message via PTY stdin: {}", sid, if msg_for_writer.chars().count() > 50 { msg_for_writer.chars().take(50).collect::<String>() } else { msg_for_writer.clone() });
            if let Err(e) = manager.write_line(&sid, &msg_for_writer) {
                log_error!("claude", "[{}] PTY write error: {}", sid, e);
                let _ = app_for_writer.emit("claude-output", ClaudeOutput {
                    session_id: sid.clone(),
                    content: format!("PTY 写入失败: {}", e),
                    thinking: String::new(),
                    done: true,
                    tool_calls: None,
                });
            }
            // Cleanup trust resolved entry
            if let Ok(mut map) = PTY_TRUST_RESOLVED.lock() {
                map.remove(&sid);
            }
        });

        Ok(())
    }

    /// Send a message via an existing PTY session's stdin.
    fn send_via_pty(app: &AppHandle, session_id: &str, message: &str) -> Result<(), String> {
        let manager = pty::get_pty_manager();
        let sid = session_id.to_string();
        let msg = message.to_string();
        let app = app.clone();

        std::thread::spawn(move || {
            if let Err(e) = manager.write_line(&sid, &msg) {
                log_error!("claude", "[{}] PTY write error: {}", sid, e);
                let _ = app.emit("claude-output", ClaudeOutput {
                    session_id: sid,
                    content: format!("PTY 写入失败: {}", e),
                    thinking: String::new(),
                    done: true,
                    tool_calls: None,
                });
            }
        });

        Ok(())
    }

    /// Send message using -p mode (original approach, for non-slash-command messages).
    fn send_via_prompt_mode(
        app: &AppHandle,
        session_id: &str,
        message: &str,
        working_dir: &str,
        claude_path: Option<&str>,
        claude_config_path: Option<&str>,
    ) -> Result<(), String> {
        let session_id_clone = session_id.to_string();
        let message = message.to_string();
        let working_dir = working_dir.to_string();
        let claude_path_owned = claude_path.map(|s| s.to_string());
        let claude_config_owned = claude_config_path.map(|s| s.to_string());
        let app = app.clone();

        let msg_preview: String = message.chars().take(100).collect();
        log_info!("claude", "[{}] Sending via -p mode (cwd: {}): {}", session_id, working_dir, msg_preview);

        let resume_session_id = {
            if let Ok(map) = CLAUDE_SESSIONS.lock() {
                map.get(session_id).cloned()
            } else {
                None
            }
        };

        if let Some(ref resume_id) = resume_session_id {
            log_info!("claude", "[{}] Resuming Claude session: {}", session_id, resume_id);
        }

        let resume_session_id_for_stdin = resume_session_id.clone();

        std::thread::spawn(move || {
            let home = if cfg!(target_os = "windows") {
                std::env::var("USERPROFILE").unwrap_or_default()
            } else {
                std::env::var("HOME").unwrap_or_default()
            };

            // Use --resume if we have a session ID, otherwise use -p with stdin
            let mut base_args = if resume_session_id.is_some() {
                vec![
                    "-p".to_string(), message.clone(),
                    "--output-format".to_string(), "stream-json".to_string(),
                    "--verbose".to_string(),
                    "--permission-mode".to_string(), "auto".to_string(),
                ]
            } else {
                // For new sessions, use stdin to pass the message
                // This avoids command line length limits and preserves newlines
                vec![
                    "--output-format".to_string(), "stream-json".to_string(),
                    "--verbose".to_string(),
                    "--permission-mode".to_string(), "auto".to_string(),
                ]
            };

            if let Some(ref resume_id) = resume_session_id {
                base_args.push("--resume".to_string());
                base_args.push(resume_id.clone());
            }

            if let Some(ref config_path) = claude_config_owned {
                if !config_path.is_empty() {
                    base_args.push("--config-file".to_string());
                    base_args.push(config_path.clone());
                }
            }

            let (cmd, args) = if let Some(ref custom_path) = claude_path_owned {
                if !custom_path.is_empty() {
                    log_info!("claude", "[{}] Using custom path: {}", session_id_clone, custom_path);
                    if cfg!(target_os = "windows") {
                        let mut a = vec!["/c".to_string(), custom_path.clone()];
                        a.extend(base_args);
                        ("cmd".to_string(), a)
                    } else {
                        (custom_path.clone(), base_args)
                    }
                } else {
                    Self::build_default_cmd(&home, base_args)
                }
            } else {
                Self::build_default_cmd(&home, base_args)
            };

            let mut path = std::env::var("PATH").unwrap_or_default();
            if cfg!(target_os = "windows") {
                let npm_bin = format!("{}\\AppData\\Roaming\\npm", home);
                if !path.contains(&npm_bin) {
                    path = format!("{};{}", npm_bin, path);
                }
            }

            log_info!("claude", "[{}] Spawning: {} {:?} (cwd: {})", session_id_clone, cmd, args, working_dir);

            let result = Command::new(&cmd)
                .args(&args)
                .current_dir(&working_dir)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .env("PATH", &path)
                .spawn();

            let mut child = match result {
                Ok(c) => {
                    log_info!("claude", "[{}] Process spawned, PID: {:?}", session_id_clone, c.id());
                    c
                }
                Err(e) => {
                    log_error!("claude", "[{}] Failed to spawn: {}", session_id_clone, e);
                    let _ = app.emit("claude-output", ClaudeOutput {
                        session_id: session_id_clone,
                        content: format!("无法启动 Claude CLI: {}", e),
                        thinking: String::new(),
                        done: true,
                        tool_calls: None,
                    });
                    return;
                }
            };

            // Write message to stdin for new sessions (not resuming)
            if resume_session_id_for_stdin.is_some() {
                // Resume mode uses -p flag, no stdin needed
            } else {
                // New session: write message to stdin
                if let Some(mut stdin) = child.stdin.take() {
                    use std::io::Write;
                    let msg_clone = message.clone();
                    std::thread::spawn(move || {
                        if let Err(e) = stdin.write_all(msg_clone.as_bytes()) {
                            log_error!("claude", "Failed to write to stdin: {}", e);
                        }
                        if let Err(e) = stdin.flush() {
                            log_error!("claude", "Failed to flush stdin: {}", e);
                        }
                    });
                }
            }

            let stdout = child.stdout.take().expect("failed to take stdout");
            let stderr = child.stderr.take().expect("failed to take stderr");

            if let Ok(mut processes) = get_child_processes().lock() {
                processes.insert(session_id_clone.clone(), child);
            }

            // Read stderr
            let sid_stderr = session_id_clone.clone();
            let app_stderr = app.clone();
            std::thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines() {
                    if let Ok(line) = line {
                        log_debug!("claude", "[{}] stderr: {}", sid_stderr, line);
                        let _ = app_stderr.emit("claude-stderr", serde_json::json!({
                            "session_id": sid_stderr,
                            "message": line,
                        }));
                    }
                }
            });

            // Read stdout and parse JSON stream
            let reader = BufReader::new(stdout);
            let mut thinking_buf = String::new();
            let mut text_buf = String::new();
            let mut tool_count = 0u32;
            let mut tool_calls_buf: Vec<ToolCall> = Vec::new();
            let mut result_received = false;

            for line in reader.lines() {
                let line = match line {
                    Ok(l) => l,
                    Err(e) => {
                        log_error!("claude", "[{}] Read error: {}", session_id_clone, e);
                        continue;
                    }
                };
                let line = line.trim();
                if line.is_empty() { continue; }

                let preview: String = line.chars().take(200).collect();
                log_debug!("claude", "[{}] stdout: {}", session_id_clone, preview);

                let parsed: serde_json::Value = match serde_json::from_str(line) {
                    Ok(v) => v,
                    Err(_) => {
                        // Not JSON — emit as output so user can see the message
                        text_buf.push_str(line);
                        text_buf.push('\n');
                        let _ = app.emit("claude-output", ClaudeOutput {
                            session_id: session_id_clone.clone(),
                            content: text_buf.clone(),
                            thinking: thinking_buf.clone(),
                            done: false,
                            tool_calls: None,
                        });
                        continue;
                    }
                };

                let msg_type = parsed.get("type").and_then(|t| t.as_str()).unwrap_or("");

                match msg_type {
                    "assistant" => {
                        if let Some(message) = parsed.get("message") {
                            if let Some(content_arr) = message.get("content").and_then(|c| c.as_array()) {
                                for block in content_arr {
                                    let block_type = block.get("type").and_then(|t| t.as_str()).unwrap_or("");
                                    match block_type {
                                        "thinking" => {
                                            if let Some(thinking) = block.get("thinking").and_then(|t| t.as_str()) {
                                                if !thinking_buf.is_empty() { thinking_buf.push_str("\n\n"); }
                                                thinking_buf.push_str(thinking);
                                                let _ = app.emit("claude-thinking", ClaudeOutput {
                                                    session_id: session_id_clone.clone(),
                                                    content: String::new(),
                                                    thinking: thinking_buf.clone(),
                                                    done: false,
                                                    tool_calls: None,
                                                });
                                            }
                                        }
                                        "text" => {
                                            if let Some(text) = block.get("text").and_then(|t| t.as_str()) {
                                                text_buf.push_str(text);
                                                let _ = app.emit("claude-output", ClaudeOutput {
                                                    session_id: session_id_clone.clone(),
                                                    content: text_buf.clone(),
                                                    thinking: thinking_buf.clone(),
                                                    done: false,
                                                    tool_calls: None,
                                                });
                                            }
                                        }
                                        "tool_use" => {
                                            tool_count += 1;
                                            let tool_id = block.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                            let tool_name = block.get("name").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
                                            let tool_input = block.get("input").cloned().unwrap_or(serde_json::Value::Null);
                                            tool_calls_buf.push(ToolCall {
                                                tool_id: tool_id.clone(),
                                                tool_name: tool_name.clone(),
                                                tool_input: tool_input.clone(),
                                            });
                                            let _ = app.emit("claude-tool-call", serde_json::json!({
                                                "session_id": session_id_clone.clone(),
                                                "tool_id": tool_id,
                                                "tool_name": tool_name,
                                                "tool_input": tool_input,
                                                "tool_number": tool_count,
                                            }));
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }
                    "result" => {
                        result_received = true;
                        let final_text = parsed.get("result").and_then(|r| r.as_str()).unwrap_or("");
                        if !final_text.is_empty() { text_buf = final_text.to_string(); }
                        let content = if text_buf.is_empty() { "Claude CLI 未返回内容".to_string() } else { text_buf.clone() };
                        log_info!("claude", "[{}] Result received ({} chars, {} tools)", session_id_clone, content.len(), tool_count);
                        let _ = app.emit("claude-output", ClaudeOutput {
                            session_id: session_id_clone.clone(),
                            content,
                            thinking: thinking_buf.clone(),
                            done: true,
                            tool_calls: if tool_calls_buf.is_empty() { None } else { Some(tool_calls_buf.clone()) },
                        });
                    }
                    "system" => {
                        if let Some(subtype) = parsed.get("subtype").and_then(|s| s.as_str()) {
                            log_info!("claude", "[{}] System event: {}", session_id_clone, subtype);
                            if subtype == "init" {
                                let sid = parsed.get("session_id").and_then(|s| s.as_str())
                                    .or_else(|| parsed.get("data").and_then(|d| d.get("session_id")).and_then(|s| s.as_str()));
                                if let Some(sid) = sid {
                                    log_info!("claude", "[{}] Claude session ID captured: {}", session_id_clone, sid);
                                    if let Ok(mut map) = CLAUDE_SESSIONS.lock() {
                                        map.insert(session_id_clone.clone(), sid.to_string());
                                    }
                                }
                            }
                        }
                    }
                    "permission" => {
                        log_info!("claude", "[{}] Permission request in -p mode: {:?}", session_id_clone, parsed);
                        // In -p mode with auto permissions, this shouldn't happen often
                        // but emit it for the UI to handle
                        let _ = app.emit("claude-permission", serde_json::json!({
                            "session_id": session_id_clone.clone(),
                            "request": parsed,
                        }));
                    }
                    _ => {
                        log_debug!("claude", "[{}] Unknown event type: {}", session_id_clone, msg_type);
                    }
                }
            }

            if !result_received {
                if text_buf.is_empty() && thinking_buf.is_empty() {
                    log_warn!("claude", "[{}] No output received from CLI", session_id_clone);
                    let _ = app.emit("claude-output", ClaudeOutput {
                        session_id: session_id_clone.clone(),
                        content: "Claude CLI 未返回内容".to_string(),
                        thinking: String::new(),
                        done: true,
                        tool_calls: None,
                    });
                } else {
                    log_warn!("claude", "[{}] Process ended without result event", session_id_clone);
                    let _ = app.emit("claude-output", ClaudeOutput {
                        session_id: session_id_clone.clone(),
                        content: text_buf.clone(),
                        thinking: thinking_buf.clone(),
                        done: true,
                        tool_calls: if tool_calls_buf.is_empty() { None } else { Some(tool_calls_buf.clone()) },
                    });
                }
            }

            if let Ok(mut processes) = get_child_processes().lock() {
                processes.remove(&session_id_clone);
            }

            log_info!("claude", "[{}] -p mode session complete", session_id_clone);
        });

        Ok(())
    }
}

impl ProviderAdapter for ClaudeAdapter {
    fn id(&self) -> &str {
        "claude"
    }

    fn name(&self) -> &str {
        "Claude Code"
    }

    fn send_message(&self, app: &AppHandle, session_id: &str, message: &str, working_dir: &str) -> Result<(), String> {
        self.send_message_with_config(app, session_id, message, working_dir, None, None)
    }

    fn send_message_with_config(
        &self,
        app: &AppHandle,
        session_id: &str,
        message: &str,
        working_dir: &str,
        provider_path: Option<&str>,
        provider_config_path: Option<&str>,
    ) -> Result<(), String> {
        // Check if this session already has an active PTY
        let has_pty = pty::get_pty_manager().has_session(session_id);

        if has_pty {
            // Reuse existing PTY session — just write the message to stdin
            let msg_preview: String = message.chars().take(100).collect();
            log_info!("claude", "[{}] Reusing PTY session, sending: {}", session_id, msg_preview);
            Self::send_via_pty(app, session_id, message)
        } else if Self::is_slash_command(message) {
            // Slash commands need PTY mode for interactive support
            log_info!("claude", "[{}] Slash command detected, starting PTY: {}", session_id, message);
            Self::start_pty_session(app, session_id, message, working_dir, provider_path, provider_config_path)
        } else {
            // Regular messages use -p mode (more reliable, supports stream-json)
            Self::send_via_prompt_mode(app, session_id, message, working_dir, provider_path, provider_config_path)
        }
    }

    fn stop_session(&self, session_id: &str) -> Result<(), String> {
        kill_session_process(session_id);
        Ok(())
    }

    fn is_available(&self) -> bool {
        let result = if cfg!(target_os = "windows") {
            let home = std::env::var("USERPROFILE").unwrap_or_default();
            let npm_claude = format!("{}\\AppData\\Roaming\\npm\\claude.cmd", home);
            if std::path::Path::new(&npm_claude).exists() {
                return true;
            }
            Command::new("cmd")
                .args(["/c", "where", "claude"])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
        } else {
            Command::new("which")
                .arg("claude")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
        };
        log_debug!("claude", "Claude CLI available: {}", result);
        result
    }
}
