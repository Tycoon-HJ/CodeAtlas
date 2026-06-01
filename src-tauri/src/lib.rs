mod commands;
mod models;
mod store;
mod provider;
pub mod pty;
pub mod file_watcher;
pub mod logger;

use provider::ProviderRegistry;
use std::sync::OnceLock;

static PROVIDER_REGISTRY: OnceLock<ProviderRegistry> = OnceLock::new();

pub fn get_provider_registry() -> &'static ProviderRegistry {
    PROVIDER_REGISTRY.get_or_init(|| ProviderRegistry::new())
}

#[tauri::command]
fn get_log_path() -> String {
    logger::get_log_path()
}

#[tauri::command]
fn get_recent_logs(count: usize) -> Vec<String> {
    logger::get_recent_logs(count)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    logger::log("INFO", "app", "CodeAtlas Studio starting up");
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_sql::Builder::new().build())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            // Greet
            commands::greet,
            // Workspace
            commands::workspace::list_workspaces,
            commands::workspace::create_workspace,
            commands::workspace::delete_workspace,
            // Project
            commands::project::list_projects,
            commands::project::create_project,
            commands::project::update_project,
            commands::project::open_project,
            commands::project::delete_project,
            // Session
            commands::session::list_sessions,
            commands::session::create_session,
            commands::session::update_session_status,
            commands::session::delete_session,
            // Message
            commands::message::list_messages,
            commands::message::create_message,
            commands::message::delete_messages,
            // Claude / Provider
            commands::claude::send_to_claude,
            commands::claude::abort_claude,
            commands::claude::list_providers,
            commands::claude::get_claude_commands,
            commands::claude::respond_permission,
            commands::claude::respond_trust_prompt,
            // Settings
            commands::settings::load_settings,
            commands::settings::save_settings,
            // File watcher
            commands::file::start_file_watcher,
            commands::file::stop_file_watcher,
            // Logging
            get_log_path,
            get_recent_logs,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
