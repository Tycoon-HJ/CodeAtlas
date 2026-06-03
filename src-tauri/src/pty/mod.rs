use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::log_info;
use crate::log_error;
use crate::log_debug;

/// Callback for raw PTY output: receives raw bytes from the PTY.
pub type RawOutputFn = Arc<dyn Fn(&[u8]) + Send + Sync>;

/// Callback for PTY exit notification.
pub type ExitFn = Arc<dyn Fn() + Send + Sync>;

/// A handle to a running PTY session.
pub struct PtySession {
    pub session_id: String,
    writer: Box<dyn Write + Send>,
    master: Box<dyn portable_pty::MasterPty + Send>,
}

impl PtySession {
    /// Write bytes to the PTY stdin.
    pub fn write(&mut self, data: &[u8]) -> std::io::Result<()> {
        self.writer.write_all(data)?;
        self.writer.flush()
    }

    /// Write a line to the PTY stdin (appends newline).
    pub fn write_line(&mut self, line: &str) -> std::io::Result<()> {
        self.write(format!("{}\n", line).as_bytes())
    }

    /// Resize the PTY.
    pub fn resize(&self, cols: u16, rows: u16) -> Result<(), String> {
        self.master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| format!("Resize failed: {}", e))
    }
}

/// Manages PTY sessions for Claude CLI processes.
pub struct PtyManager {
    sessions: Mutex<HashMap<String, Arc<Mutex<PtySession>>>>,
}

impl PtyManager {
    pub fn new() -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
        }
    }

    /// Build a CommandBuilder for claude, handling Windows path resolution.
    fn build_claude_cmd(
        claude_path: Option<&str>,
        working_dir: &str,
        args: &[String],
    ) -> CommandBuilder {
        if let Some(custom_path) = claude_path {
            if !custom_path.is_empty() {
                let mut cmd = CommandBuilder::new(custom_path);
                cmd.cwd(working_dir);
                for arg in args {
                    cmd.arg(arg);
                }
                return cmd;
            }
        }

        // On Windows, use cmd /c claude.cmd to avoid POSIX shell script issue
        if cfg!(target_os = "windows") {
            let home = std::env::var("USERPROFILE").unwrap_or_default();
            let npm_claude = format!("{}\\AppData\\Roaming\\npm\\claude.cmd", home);
            let mut cmd = if std::path::Path::new(&npm_claude).exists() {
                let mut c = CommandBuilder::new("cmd");
                c.arg("/c");
                c.arg(&npm_claude);
                c
            } else {
                let mut c = CommandBuilder::new("cmd");
                c.arg("/c");
                c.arg("claude");
                c
            };
            cmd.cwd(working_dir);
            for arg in args {
                cmd.arg(arg);
            }
            // Ensure PATH includes npm bin
            let path = std::env::var("PATH").unwrap_or_default();
            let npm_bin = format!("{}\\AppData\\Roaming\\npm", home);
            if !path.contains(&npm_bin) {
                cmd.env("PATH", format!("{};{}", npm_bin, path));
            }
            cmd
        } else {
            let mut cmd = CommandBuilder::new("claude");
            cmd.cwd(working_dir);
            for arg in args {
                cmd.arg(arg);
            }
            let home = dirs_next::home_dir().unwrap_or_default();
            let path = std::env::var("PATH").unwrap_or_default();
            let npm_bin = home.join(".npm/bin").to_string_lossy().to_string();
            if !path.contains(&npm_bin) {
                cmd.env("PATH", format!("{}:{}", npm_bin, path));
            }
            cmd
        }
    }

    /// Spawn a new Claude CLI process in a PTY (JSON line-buffered mode).
    ///
    /// `session_id` — app-level session ID for tracking
    /// `claude_path` — path to claude binary (or None for default)
    /// `working_dir` — working directory for the process
    /// `args` — extra CLI args (e.g. `--resume`, `--output-format stream-json`)
    /// `output_handler` — callback invoked with each line of stdout
    pub fn spawn<F>(
        &self,
        session_id: &str,
        claude_path: Option<&str>,
        working_dir: &str,
        args: Vec<String>,
        output_handler: F,
    ) -> Result<(), String>
    where
        F: Fn(String) + Send + 'static,
    {
        let pty_system = native_pty_system();

        let pty = pty_system
            .openpty(PtySize {
                rows: 40,
                cols: 120,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| format!("Failed to open PTY: {}", e))?;

        let cmd = Self::build_claude_cmd(claude_path, working_dir, &args);

        log_info!("pty", "[{}] Spawning (json mode) in: {}", session_id, working_dir);

        let mut child = pty
            .slave
            .spawn_command(cmd)
            .map_err(|e| format!("Failed to spawn command: {}", e))?;

        let writer = pty
            .master
            .take_writer()
            .map_err(|e| format!("Failed to take writer: {}", e))?;

        let reader = pty
            .master
            .try_clone_reader()
            .map_err(|e| format!("Failed to clone reader: {}", e))?;

        // Store session handle (keep master for resize support)
        let session = Arc::new(Mutex::new(PtySession {
            session_id: session_id.to_string(),
            writer,
            master: pty.master,
        }));

        if let Ok(mut sessions) = self.sessions.lock() {
            sessions.insert(session_id.to_string(), session.clone());
        }

        // Read stdout in a background thread
        let sid = session_id.to_string();
        thread::spawn(move || {
            let mut reader = reader;
            let mut buf = [0u8; 8192];
            let mut line_buf = String::new();

            loop {
                match reader.read(&mut buf) {
                    Ok(0) => {
                        log_info!("pty", "[{}] EOF reached", sid);
                        break;
                    }
                    Ok(n) => {
                        let chunk = String::from_utf8_lossy(&buf[..n]);
                        line_buf.push_str(&chunk);

                        // Process complete lines
                        while let Some(pos) = line_buf.find('\n') {
                            let line = line_buf[..pos].trim().to_string();
                            line_buf = line_buf[pos + 1..].to_string();
                            if !line.is_empty() {
                                let preview: String = line.chars().take(200).collect();
                                log_debug!("pty", "[{}] stdout: {}", sid, preview);
                                output_handler(line);
                            }
                        }
                    }
                    Err(e) => {
                        log_error!("pty", "[{}] Read error: {}", sid, e);
                        break;
                    }
                }
            }

            // Process any remaining data in line_buf
            let remaining = line_buf.trim().to_string();
            if !remaining.is_empty() {
                output_handler(remaining);
            }

            // Remove session from map when reader exits (process died)
            get_pty_manager().close(&sid);
            log_info!("pty", "[{}] Reader thread exited, session cleaned up", sid);
        });

        // Wait for child exit in background
        let sid = session_id.to_string();
        thread::spawn(move || {
            let status = child.wait();
            log_info!("pty", "[{}] Process exited: {:?}", sid, status);
        });

        log_info!("pty", "[{}] PTY session started", session_id);
        Ok(())
    }

    /// Spawn a new process in raw PTY mode (for interactive terminal).
    ///
    /// Unlike `spawn()`, this emits raw bytes via `output_fn` and does NOT parse JSON.
    /// The process is spawned without `--output-format stream-json` so the CLI renders
    /// its full interactive TUI.
    pub fn spawn_raw(
        &self,
        session_id: &str,
        claude_path: Option<&str>,
        working_dir: &str,
        args: Vec<String>,
        cols: u16,
        rows: u16,
        output_fn: RawOutputFn,
        exit_fn: ExitFn,
    ) -> Result<(), String> {
        let pty_system = native_pty_system();

        let pty = pty_system
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| format!("Failed to open PTY: {}", e))?;

        let cmd = Self::build_claude_cmd(claude_path, working_dir, &args);

        log_info!("pty", "[{}] Spawning (raw mode) in: {} ({}x{})", session_id, working_dir, cols, rows);

        let mut child = pty
            .slave
            .spawn_command(cmd)
            .map_err(|e| format!("Failed to spawn command: {}", e))?;

        let writer = pty
            .master
            .take_writer()
            .map_err(|e| format!("Failed to take writer: {}", e))?;

        let reader = pty
            .master
            .try_clone_reader()
            .map_err(|e| format!("Failed to clone reader: {}", e))?;

        let session = Arc::new(Mutex::new(PtySession {
            session_id: session_id.to_string(),
            writer,
            master: pty.master,
        }));

        if let Ok(mut sessions) = self.sessions.lock() {
            sessions.insert(session_id.to_string(), session.clone());
        }

        // Read raw bytes in background thread
        let sid = session_id.to_string();
        thread::spawn(move || {
            let mut reader = reader;
            let mut buf = [0u8; 16384];

            loop {
                match reader.read(&mut buf) {
                    Ok(0) => {
                        log_info!("pty", "[{}] Raw EOF reached", sid);
                        break;
                    }
                    Ok(n) => {
                        output_fn(&buf[..n]);
                    }
                    Err(e) => {
                        log_error!("pty", "[{}] Raw read error: {}", sid, e);
                        break;
                    }
                }
            }

            exit_fn();
            get_pty_manager().close(&sid);
            log_info!("pty", "[{}] Raw reader thread exited", sid);
        });

        // Wait for child exit in background
        let sid = session_id.to_string();
        thread::spawn(move || {
            let status = child.wait();
            log_info!("pty", "[{}] Raw process exited: {:?}", sid, status);
        });

        log_info!("pty", "[{}] Raw PTY session started", session_id);
        Ok(())
    }

    /// Resize a PTY session.
    pub fn resize(&self, session_id: &str, cols: u16, rows: u16) -> Result<(), String> {
        let sessions = self.sessions.lock().map_err(|e| format!("Lock error: {}", e))?;
        let session = sessions
            .get(session_id)
            .ok_or_else(|| format!("Session {} not found", session_id))?;
        let session = session.lock().map_err(|e| format!("Lock error: {}", e))?;
        session.resize(cols, rows)
    }

    /// Write a line to a session's stdin.
    pub fn write_line(&self, session_id: &str, line: &str) -> Result<(), String> {
        let sessions = self.sessions.lock().map_err(|e| format!("Lock error: {}", e))?;
        let session = sessions
            .get(session_id)
            .ok_or_else(|| format!("Session {} not found", session_id))?;
        let mut session = session.lock().map_err(|e| format!("Lock error: {}", e))?;
        session
            .write_line(line)
            .map_err(|e| format!("Write error: {}", e))
    }

    /// Write raw bytes to a session's stdin.
    pub fn write(&self, session_id: &str, data: &[u8]) -> Result<(), String> {
        let sessions = self.sessions.lock().map_err(|e| format!("Lock error: {}", e))?;
        let session = sessions
            .get(session_id)
            .ok_or_else(|| format!("Session {} not found", session_id))?;
        let mut session = session.lock().map_err(|e| format!("Lock error: {}", e))?;
        session
            .write(data)
            .map_err(|e| format!("Write error: {}", e))
    }

    /// Close and remove a PTY session.
    pub fn close(&self, session_id: &str) {
        if let Ok(mut sessions) = self.sessions.lock() {
            sessions.remove(session_id);
            log_info!("pty", "[{}] Session closed", session_id);
        }
    }

    /// Check if a session exists.
    pub fn has_session(&self, session_id: &str) -> bool {
        self.sessions
            .lock()
            .map(|s| s.contains_key(session_id))
            .unwrap_or(false)
    }
}

/// Global PTY manager instance.
static PTY_MANAGER: once_cell::sync::Lazy<PtyManager> =
    once_cell::sync::Lazy::new(|| PtyManager::new());

pub fn get_pty_manager() -> &'static PtyManager {
    &PTY_MANAGER
}
