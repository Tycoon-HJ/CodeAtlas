pub mod workspace;
pub mod project;
pub mod session;
pub mod message;
pub mod claude;
pub mod file;
pub mod settings;

use serde::Serialize;

#[derive(Serialize)]
pub struct GreetResponse {
    pub message: String,
}

#[tauri::command]
pub fn greet(name: &str) -> GreetResponse {
    GreetResponse {
        message: format!("Hello, {}! Welcome to CodeAtlas Studio.", name),
    }
}
