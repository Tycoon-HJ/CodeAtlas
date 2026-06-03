use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter};

use super::ProviderAdapter;
use crate::log_debug;
use crate::log_error;
use crate::log_info;
use crate::models::{ClaudeOutput, ToolCall};

type ChildProcesses = Mutex<HashMap<String, Child>>;
static CHILD_PROCESSES: once_cell::sync::Lazy<ChildProcesses> =
    once_cell::sync::Lazy::new(|| Mutex::new(HashMap::new()));

/// Maps app session_id -> Codex CLI session_id for session continuity.
type CodexSessionMap = Mutex<HashMap<String, String>>;
static CODEX_SESSIONS: once_cell::sync::Lazy<CodexSessionMap> =
    once_cell::sync::Lazy::new(|| Mutex::new(HashMap::new()));

/// Clear stored Codex session ID for an app session.
fn clear_codex_session(session_id: &str) {
    if let Ok(mut map) = CODEX_SESSIONS.lock() {
        map.remove(session_id);
    }
}

/// Get stored Codex session ID for an app session.
fn get_codex_session(session_id: &str) -> Option<String> {
    if let Ok(map) = CODEX_SESSIONS.lock() {
        map.get(session_id).cloned()
    } else {
        None
    }
}

/// Store Codex session ID for an app session.
fn set_codex_session(app_session_id: &str, codex_session_id: &str) {
    log_info!("codex", "[{}] Storing Codex session ID: {}", app_session_id, codex_session_id);
    if let Ok(mut map) = CODEX_SESSIONS.lock() {
        map.insert(app_session_id.to_string(), codex_session_id.to_string());
    }
}

pub fn kill_session_process(session_id: &str) {
    if let Ok(mut processes) = CHILD_PROCESSES.lock() {
        if let Some(mut child) = processes.remove(session_id) {
            let _ = child.kill();
            log_info!("codex", "Killed process for session {}", session_id);
        }
    }
    clear_codex_session(session_id);
}

pub struct CodexAdapter;

impl CodexAdapter {
    fn build_default_cmd(home: &str, base_args: Vec<String>) -> (String, Vec<String>) {
        if cfg!(target_os = "windows") {
            let npm_codex = format!("{}\\AppData\\Roaming\\npm\\codex.cmd", home);
            if std::path::Path::new(&npm_codex).exists() {
                let mut a = vec!["/c".to_string(), npm_codex];
                a.extend(base_args);
                ("cmd".to_string(), a)
            } else {
                let mut a = vec!["/c".to_string(), "codex".to_string()];
                a.extend(base_args);
                ("cmd".to_string(), a)
            }
        } else {
            ("codex".to_string(), base_args)
        }
    }
}

impl ProviderAdapter for CodexAdapter {
    fn id(&self) -> &str {
        "codex"
    }

    fn name(&self) -> &str {
        "Codex CLI"
    }

    fn send_message(
        &self,
        app: &AppHandle,
        session_id: &str,
        message: &str,
        working_dir: &str,
    ) -> Result<(), String> {
        self.send_message_with_config(app, session_id, message, working_dir, None, None)
    }

    fn send_message_with_config(
        &self,
        app: &AppHandle,
        session_id: &str,
        message: &str,
        working_dir: &str,
        provider_path: Option<&str>,
        _provider_config_path: Option<&str>,
    ) -> Result<(), String> {
        let session_id_clone = session_id.to_string();
        let message = message.to_string();
        let working_dir = working_dir.to_string();
        let provider_path_owned = provider_path.map(|s| s.to_string());
        let app = app.clone();

        // Check for existing Codex session to resume
        let resume_id = get_codex_session(session_id);

        let msg_preview: String = message.chars().take(100).collect();
        log_info!("codex", "[{}] Sending via exec (cwd: {}): {}, resume_id={:?}", session_id, working_dir, msg_preview, resume_id);

        std::thread::spawn(move || {
            let home = if cfg!(target_os = "windows") {
                std::env::var("USERPROFILE").unwrap_or_default()
            } else {
                std::env::var("HOME").unwrap_or_default()
            };

            // Build args based on whether we're resuming or starting fresh
            // Note: `codex exec resume` does NOT support --sandbox flag
            let base_args = if let Some(ref codex_sid) = resume_id {
                log_info!("codex", "[{}] Resuming Codex session: {}", session_id_clone, codex_sid);
                vec![
                    "exec".to_string(),
                    "resume".to_string(),
                    codex_sid.clone(),
                    message.clone(),
                    "--json".to_string(),
                    "--skip-git-repo-check".to_string(),
                ]
            } else {
                vec![
                    "exec".to_string(),
                    message.clone(),
                    "--sandbox".to_string(),
                    "danger-full-access".to_string(),
                    "--json".to_string(),
                    "--skip-git-repo-check".to_string(),
                ]
            };

            let (cmd, args) = if let Some(ref custom_path) = provider_path_owned {
                if !custom_path.is_empty() {
                    log_info!("codex", "[{}] Using custom path: {}", session_id_clone, custom_path);
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

            log_info!("codex", "[{}] Spawning: {} {:?} (cwd: {})", session_id_clone, cmd, args, working_dir);

            let result = Command::new(&cmd)
                .args(&args)
                .current_dir(&working_dir)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .env("PATH", &path)
                .spawn();

            let mut child = match result {
                Ok(c) => {
                    log_info!("codex", "[{}] Process spawned, PID: {:?}", session_id_clone, c.id());
                    c
                }
                Err(e) => {
                    log_error!("codex", "[{}] Failed to spawn: {}", session_id_clone, e);
                    let _ = app.emit("claude-output", ClaudeOutput {
                        session_id: session_id_clone,
                        content: format!("Unable to start Codex CLI: {}", e),
                        thinking: String::new(),
                        done: true,
                        tool_calls: None,
                    });
                    return;
                }
            };

            let stdout = child.stdout.take().expect("failed to take stdout");
            let stderr = child.stderr.take().expect("failed to take stderr");

            if let Ok(mut processes) = CHILD_PROCESSES.lock() {
                processes.insert(session_id_clone.clone(), child);
            }

            // Collect stderr in background
            let stderr_lines: std::sync::Arc<std::sync::Mutex<Vec<String>>> =
                std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
            let stderr_lines2 = stderr_lines.clone();
            let sid_stderr = session_id_clone.clone();
            let app_stderr = app.clone();
            std::thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines() {
                    if let Ok(line) = line {
                        log_debug!("codex", "[{}] stderr: {}", sid_stderr, line);
                        if let Ok(mut lines) = stderr_lines2.lock() {
                            lines.push(line.clone());
                        }
                        let _ = app_stderr.emit("claude-stderr", serde_json::json!({
                            "session_id": sid_stderr,
                            "message": line,
                        }));
                    }
                }
            });

            // Read stdout — parse Codex JSONL events
            // Codex event format:
            //   {"type":"item.completed","item":{"id":"...","type":"reasoning","text":"..."}}
            //   {"type":"item.completed","item":{"id":"...","type":"agent_message","text":"..."}}
            //   {"type":"turn.completed","usage":{...}}
            //   {"type":"session.created","session_id":"..."}
            let reader = BufReader::new(stdout);
            let mut text_buf = String::new();
            let mut thinking_buf = String::new();
            let mut tool_calls_buf: Vec<ToolCall> = Vec::new();
            let mut got_content = false;
            let mut done_emitted = false; // Track whether we already sent done:true

            for line in reader.lines() {
                let line = match line {
                    Ok(l) => l,
                    Err(e) => {
                        log_error!("codex", "[{}] Read error: {}", session_id_clone, e);
                        continue;
                    }
                };

                log_debug!("codex", "[{}] stdout: {}", session_id_clone, line);

                if let Ok(event) = serde_json::from_str::<serde_json::Value>(&line) {
                    let etype = event.get("type").and_then(|t| t.as_str()).unwrap_or("");

                    match etype {
                        "thread.started" => {
                            // Capture Codex thread ID for future resume
                            if let Some(tid) = event.get("thread_id").and_then(|s| s.as_str()) {
                                log_info!("codex", "[{}] Codex thread started: {}", session_id_clone, tid);
                                set_codex_session(&session_id_clone, tid);
                            }
                        }
                        "item.completed" => {
                            if let Some(item) = event.get("item") {
                                let item_type = item.get("type").and_then(|t| t.as_str()).unwrap_or("");
                                let text = item.get("text").and_then(|t| t.as_str()).unwrap_or("");

                                match item_type {
                                    "reasoning" => {
                                        if !text.is_empty() {
                                            thinking_buf.push_str(text);
                                            let _ = app.emit("claude-thinking", ClaudeOutput {
                                                session_id: session_id_clone.clone(),
                                                content: String::new(),
                                                thinking: thinking_buf.clone(),
                                                done: false,
                                                tool_calls: None,
                                            });
                                        }
                                    }
                                    "agent_message" => {
                                        if !text.is_empty() {
                                            text_buf.push_str(text);
                                            got_content = true;
                                            let _ = app.emit("claude-output", ClaudeOutput {
                                                session_id: session_id_clone.clone(),
                                                content: text_buf.clone(),
                                                thinking: thinking_buf.clone(),
                                                done: false,
                                                tool_calls: None,
                                            });
                                        }
                                    }
                                    "tool_use" | "tool_call" => {
                                        let tool_name = item.get("name").and_then(|n| n.as_str()).unwrap_or("unknown");
                                        let tool_input = item.get("input").cloned().unwrap_or(serde_json::Value::Null);
                                        tool_calls_buf.push(ToolCall {
                                            tool_id: item.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                                            tool_name: tool_name.to_string(),
                                            tool_input: tool_input.clone(),
                                        });
                                        let _ = app.emit("claude-tool-call", serde_json::json!({
                                            "session_id": session_id_clone.clone(),
                                            "tool_id": item.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                                            "tool_name": tool_name,
                                            "tool_input": tool_input,
                                            "tool_number": 0,
                                        }));
                                    }
                                    _ => {
                                        // Other item types — try to extract text
                                        if !text.is_empty() {
                                            text_buf.push_str(text);
                                            got_content = true;
                                            let _ = app.emit("claude-output", ClaudeOutput {
                                                session_id: session_id_clone.clone(),
                                                content: text_buf.clone(),
                                                thinking: thinking_buf.clone(),
                                                done: false,
                                                tool_calls: None,
                                            });
                                        }
                                    }
                                }
                            }
                        }
                        "turn.completed" => {
                            // Turn finished — emit done signal immediately so frontend stops rendering
                            log_info!("codex", "[{}] Turn completed, emitting done signal", session_id_clone);
                            let content = if got_content && !text_buf.is_empty() {
                                text_buf.clone()
                            } else {
                                "Codex CLI 未返回内容".to_string()
                            };
                            let _ = app.emit("claude-output", ClaudeOutput {
                                session_id: session_id_clone.clone(),
                                content,
                                thinking: thinking_buf.clone(),
                                done: true,
                                tool_calls: if tool_calls_buf.is_empty() { None } else { Some(tool_calls_buf.clone()) },
                            });
                            done_emitted = true;
                        }
                        _ => {
                            // Other event types — try generic text extraction
                            let text = event.get("text").and_then(|t| t.as_str())
                                .or_else(|| event.get("content").and_then(|t| t.as_str()));
                            if let Some(text) = text {
                                if !text.is_empty() {
                                    text_buf.push_str(text);
                                    got_content = true;
                                }
                            }
                        }
                    }
                } else if !line.is_empty() {
                    // Not JSON — accumulate as plain text
                    if !text_buf.is_empty() {
                        text_buf.push('\n');
                    }
                    text_buf.push_str(&line);
                    got_content = true;
                    let _ = app.emit("claude-output", ClaudeOutput {
                        session_id: session_id_clone.clone(),
                        content: text_buf.clone(),
                        thinking: String::new(),
                        done: false,
                        tool_calls: None,
                    });
                }
            }

            // Wait for exit
            if let Ok(mut processes) = CHILD_PROCESSES.lock() {
                processes.remove(&session_id_clone);
            }

            // Only emit final result if we haven't already sent done:true via turn.completed
            if !done_emitted {
                let content = if got_content && !text_buf.is_empty() {
                    text_buf
                } else {
                    let stderr_text = stderr_lines.lock()
                        .map(|lines| {
                            lines.iter()
                                .filter(|l| !l.contains("WARNING:") && !l.contains("node.exe"))
                                .cloned()
                                .collect::<Vec<_>>()
                                .join("\n")
                        })
                        .unwrap_or_default();
                    if !stderr_text.is_empty() {
                        format!("Codex CLI error:\n{}", stderr_text)
                    } else {
                        "Codex CLI did not return content".to_string()
                    }
                };

                log_info!("codex", "[{}] Complete ({} chars, got_content={})", session_id_clone, content.len(), got_content);
                let _ = app.emit("claude-output", ClaudeOutput {
                    session_id: session_id_clone.clone(),
                    content,
                    thinking: thinking_buf,
                    done: true,
                    tool_calls: if tool_calls_buf.is_empty() { None } else { Some(tool_calls_buf.clone()) },
                });
            }

            log_info!("codex", "[{}] Session complete", session_id_clone);
        });

        Ok(())
    }

    fn stop_session(&self, session_id: &str) -> Result<(), String> {
        kill_session_process(session_id);
        Ok(())
    }

    fn is_available(&self) -> bool {
        let result = if cfg!(target_os = "windows") {
            let home = std::env::var("USERPROFILE").unwrap_or_default();
            let npm_codex = format!("{}\\AppData\\Roaming\\npm\\codex.cmd", home);
            if std::path::Path::new(&npm_codex).exists() {
                return true;
            }
            Command::new("cmd")
                .args(["/c", "where", "codex"])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
        } else {
            Command::new("which")
                .arg("codex")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
        };
        log_debug!("codex", "Codex CLI available: {}", result);
        result
    }
}
