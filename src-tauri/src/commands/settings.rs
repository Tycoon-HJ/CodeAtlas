use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::store;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub theme: String,
    pub locale: String,
    pub git_user_name: String,
    pub git_user_email: String,
    pub proxy_url: String,
    pub log_level: String,
    pub claude_path: String,
    pub claude_config_path: String,
    pub codex_path: String,
    pub codex_config_path: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: "dark".into(),
            locale: "zh-CN".into(),
            git_user_name: String::new(),
            git_user_email: String::new(),
            proxy_url: String::new(),
            log_level: "info".into(),
            claude_path: String::new(),
            claude_config_path: String::new(),
            codex_path: String::new(),
            codex_config_path: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentState {
    pub enabled_providers: Vec<String>,
}

impl Default for AgentState {
    fn default() -> Self {
        Self {
            enabled_providers: vec!["claude".into()],
        }
    }
}

#[tauri::command]
pub fn load_settings(app: AppHandle) -> AppSettings {
    store::load_json(&app, "settings").unwrap_or_default()
}

#[tauri::command]
pub fn save_settings(app: AppHandle, settings: AppSettings) -> Result<(), String> {
    store::save_json(&app, "settings", &settings);
    Ok(())
}

#[tauri::command]
pub fn load_agent_state(app: AppHandle) -> AgentState {
    store::load_json(&app, "agent_state").unwrap_or_default()
}

#[tauri::command]
pub fn save_agent_state(app: AppHandle, state: AgentState) -> Result<(), String> {
    store::save_json(&app, "agent_state", &state);
    Ok(())
}
