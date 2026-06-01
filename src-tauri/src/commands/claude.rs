use serde::Serialize;
use serde_json::json;
use tauri::AppHandle;

use crate::get_provider_registry;
use crate::log_info;
use crate::log_error;
use crate::provider::claude_adapter;
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
    Ok(())
}

/// Return known Claude CLI slash commands.
/// Claude CLI's -p mode does not support slash commands, so we return a static list.
/// The frontend registry handles command execution by translating to prompts.
#[tauri::command]
pub fn get_claude_commands(_session_id: String) -> serde_json::Value {
    log_info!("cmd", "get_claude_commands");
    let commands = vec![
        json!({ "name": "/help", "icon": "❓", "description": "显示帮助信息", "usage": "/help" }),
        json!({ "name": "/compact", "icon": "🗜️", "description": "压缩对话上下文以节省 token", "usage": "/compact [说明]" }),
        json!({ "name": "/clear", "icon": "🗑️", "description": "清空当前对话历史", "usage": "/clear" }),
        json!({ "name": "/cost", "icon": "💰", "description": "显示当前会话信息", "usage": "/cost" }),
        json!({ "name": "/model", "icon": "🤖", "description": "切换或查看当前 AI 模型", "usage": "/model [模型名]" }),
        json!({ "name": "/config", "icon": "⚙️", "description": "查看或修改配置", "usage": "/config" }),
        json!({ "name": "/memory", "icon": "🧠", "description": "管理持久记忆", "usage": "/memory" }),
        json!({ "name": "/permissions", "icon": "🔐", "description": "查看或修改权限设置", "usage": "/permissions" }),
        json!({ "name": "/status", "icon": "📊", "description": "显示当前会话状态", "usage": "/status" }),
        json!({ "name": "/doctor", "icon": "🩺", "description": "诊断 Claude CLI 环境问题", "usage": "/doctor" }),
        json!({ "name": "/login", "icon": "🔑", "description": "登录 Anthropic 账号", "usage": "/login" }),
        json!({ "name": "/logout", "icon": "🚪", "description": "退出登录", "usage": "/logout" }),
        json!({ "name": "/terminal-setup", "icon": "💻", "description": "配置终端集成", "usage": "/terminal-setup" }),
        json!({ "name": "/vim", "icon": "📝", "description": "切换 Vim 编辑模式", "usage": "/vim" }),
        json!({ "name": "/mcp", "icon": "🔌", "description": "管理 MCP 服务器", "usage": "/mcp" }),
        json!({ "name": "/init", "icon": "📋", "description": "初始化项目配置文件", "usage": "/init" }),
        json!({ "name": "/review", "icon": "🔍", "description": "代码审查", "usage": "/review [目标]" }),
        json!({ "name": "/pr-comments", "icon": "💬", "description": "查看 PR 评论", "usage": "/pr-comments" }),
    ];
    json!({ "commands": commands })
}

/// Respond to a permission request from Claude.
/// Sends 'y' (allow) or 'n' (deny) to the PTY stdin.
#[tauri::command]
pub fn respond_permission(session_id: String, allow: bool) -> Result<(), String> {
    let response = if allow { "y" } else { "n" };
    log_info!("cmd", "respond_permission: session={}, allow={}", session_id, allow);
    let manager = pty::get_pty_manager();
    manager.write_line(&session_id, response)
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
