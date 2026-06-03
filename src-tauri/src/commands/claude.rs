use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::get_provider_registry;
use crate::log_info;
use crate::log_error;
use crate::provider::claude_adapter;
use crate::provider::codex_adapter;
use crate::pty;

#[derive(Serialize)]
pub struct ProviderInfo {
    pub id: String,
    pub name: String,
    pub available: bool,
}

#[tauri::command]
pub fn send_to_claude(
    app: AppHandle,
    session_id: String,
    message: String,
    working_dir: String,
    claude_path: Option<String>,
    claude_config_path: Option<String>,
) -> Result<(), String> {
    log_info!("cmd", "send_to_claude: session={}, cwd={}, msg_len={}, path={:?}, config={:?}",
        session_id, working_dir, message.len(), claude_path, claude_config_path);
    let registry = get_provider_registry();
    let adapter = registry.get("claude")
        .ok_or_else(|| {
            log_error!("cmd", "Claude provider not found");
            "Claude provider not found".to_string()
        })?;
    adapter.send_message_with_config(&app, &session_id, &message, &working_dir, claude_path.as_deref(), claude_config_path.as_deref())
}

#[tauri::command]
pub fn send_message(
    app: AppHandle,
    provider_id: String,
    session_id: String,
    message: String,
    working_dir: String,
    provider_path: Option<String>,
    provider_config_path: Option<String>,
) -> Result<(), String> {
    log_info!("cmd", "send_message: provider={}, session={}, cwd={}, msg_len={}, path={:?}, config={:?}",
        provider_id, session_id, working_dir, message.len(), provider_path, provider_config_path);
    let registry = get_provider_registry();
    let adapter = registry.get(&provider_id)
        .ok_or_else(|| {
            log_error!("cmd", "Provider '{}' not found", provider_id);
            format!("Provider '{}' not found", provider_id)
        })?;
    adapter.send_message_with_config(&app, &session_id, &message, &working_dir, provider_path.as_deref(), provider_config_path.as_deref())
}

#[tauri::command]
pub fn list_providers() -> Vec<ProviderInfo> {
    log_info!("cmd", "list_providers");
    let registry = get_provider_registry();
    let all_ids = ["claude", "codex", "gemini", "qwen", "opencode"];
    all_ids.iter().map(|&id| {
        let adapter = registry.get(id);
        ProviderInfo {
            id: id.to_string(),
            name: adapter.map(|a| a.name().to_string()).unwrap_or_else(|| id.to_string()),
            available: adapter.map(|a| a.is_available()).unwrap_or(false),
        }
    }).collect()
}

#[tauri::command]
pub fn abort_claude(session_id: String) -> Result<(), String> {
    log_info!("cmd", "abort_claude: session={}", session_id);
    claude_adapter::kill_session_process(&session_id);
    codex_adapter::kill_session_process(&session_id);
    Ok(())
}

/// Respond to a permission request from Claude.
/// Sends 'y' (allow) or 'n' (deny) to the PTY stdin.
#[tauri::command]
pub fn respond_permission(session_id: String, allow: bool) -> Result<(), String> {
    let response = if allow { "y" } else { "n" };
    log_info!("cmd", "respond_permission: session={}, allow={}", session_id, allow);

    // Try PTY mode first
    let manager = pty::get_pty_manager();
    if manager.has_session(&session_id) {
        return manager.write_line(&session_id, response);
    }

    // In -p mode with auto permissions, this shouldn't be called
    // but return OK to avoid errors
    log_info!("cmd", "respond_permission: no PTY session found, ignoring (auto mode)");
    Ok(())
}

#[tauri::command]
pub fn respond_trust_prompt(session_id: String, trust: bool) -> Result<(), String> {
    let response = if trust { "1" } else { "2" };
    log_info!("cmd", "respond_trust_prompt: session={}, trust={}", session_id, trust);

    // Mark trust as resolved so the message sender can proceed
    if let Ok(map) = claude_adapter::PTY_TRUST_RESOLVED.lock() {
        if let Some(resolved) = map.get(&session_id) {
            if let Ok(mut r) = resolved.lock() {
                *r = true;
            }
        }
    }

    let manager = pty::get_pty_manager();
    manager.write_line(&session_id, response)
}

// ─── Interactive Terminal Commands ───

/// Spawn Claude CLI in raw interactive PTY mode (full TUI, all slash commands work).
#[tauri::command]
pub fn spawn_terminal(
    app: AppHandle,
    session_id: String,
    working_dir: String,
    claude_path: Option<String>,
    cols: u16,
    rows: u16,
) -> Result<(), String> {
    log_info!("cmd", "spawn_terminal: session={}, cwd={}, {}x{}", session_id, working_dir, cols, rows);

    let manager = pty::get_pty_manager();

    // Don't spawn if already exists
    if manager.has_session(&session_id) {
        log_info!("cmd", "[{}] Terminal session already exists, reusing", session_id);
        return Ok(());
    }

    let args = vec![
        "--permission-mode".to_string(), "default".to_string(),
    ];

    let path_ref = claude_path.as_deref();
    let sid = session_id.clone();
    let app_clone = app.clone();

    // Raw output: emit bytes directly to frontend
    let output_fn: pty::RawOutputFn = std::sync::Arc::new(move |data: &[u8]| {
        // Base64-encode raw bytes for safe transport over JSON events
        use base64::Engine;
        let encoded = base64::engine::general_purpose::STANDARD.encode(data);
        let _ = app_clone.emit(&format!("pty-data-{}", sid), serde_json::json!({
            "data": encoded,
        }));
    });

    let sid2 = session_id.clone();
    let app_clone2 = app.clone();
    let exit_fn: pty::ExitFn = std::sync::Arc::new(move || {
        let _ = app_clone2.emit(&format!("pty-exit-{}", sid2), serde_json::json!({}));
    });

    manager.spawn_raw(&session_id, path_ref, &working_dir, args, cols, rows, output_fn, exit_fn)?;

    log_info!("cmd", "[{}] Interactive terminal spawned", session_id);
    Ok(())
}

/// Write raw bytes to an interactive terminal session's stdin.
#[tauri::command]
pub fn write_terminal(session_id: String, data: String) -> Result<(), String> {
    let manager = pty::get_pty_manager();
    // Data is base64-encoded from frontend
    use base64::Engine;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&data)
        .map_err(|e| format!("Base64 decode error: {}", e))?;
    manager.write(&session_id, &bytes)
}

/// Resize an interactive terminal session.
#[tauri::command]
pub fn resize_terminal(session_id: String, cols: u16, rows: u16) -> Result<(), String> {
    let manager = pty::get_pty_manager();
    manager.resize(&session_id, cols, rows)
}

/// Close an interactive terminal session.
#[tauri::command]
pub fn close_terminal(session_id: String) -> Result<(), String> {
    log_info!("cmd", "close_terminal: session={}", session_id);
    let manager = pty::get_pty_manager();
    manager.close(&session_id);
    Ok(())
}
