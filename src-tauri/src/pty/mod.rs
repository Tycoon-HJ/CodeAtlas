use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::log_info;
use crate::log_error;
use crate::log_debug;

/// A handle to a running PTY session.
pub struct PtySession {
    pub session_id: String,
    writer: Box<dyn Write + Send>,
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

    /// Spawn a new Claude CLI process in a PTY.
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

        let cmd_name = claude_path.unwrap_or("claude");
        let mut cmd = CommandBuilder::new(cmd_name);
        cmd.cwd(working_dir);
        for arg in &args {
            cmd.arg(arg);
        }

        // Ensure PATH includes common install locations
        let home = dirs_next::home_dir().unwrap_or_default();
        let path = std::env::var("PATH").unwrap_or_default();
        let npm_bin = home.join(".npm/bin").to_string_lossy().to_string();
        let new_path = if path.contains(&npm_bin) {
            path
        } else {
            format!("{}:{}", npm_bin, path)
        };
        cmd.env("PATH", &new_path);

        log_info!("pty", "[{}] Spawning: {} {:?} (cwd: {})", session_id, cmd_name, args, working_dir);

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

        // Store session handle
        let session = Arc::new(Mutex::new(PtySession {
            session_id: session_id.to_string(),
            writer,
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
