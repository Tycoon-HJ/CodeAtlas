use crate::file_watcher;
use tauri::AppHandle;

#[tauri::command]
pub fn start_file_watcher(app: AppHandle, path: String) -> Result<(), String> {
    file_watcher::start_watcher(app, path)
}

#[tauri::command]
pub fn stop_file_watcher() -> Result<(), String> {
    file_watcher::stop_watcher()
}
