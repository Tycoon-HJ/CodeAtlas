use tauri::AppHandle;

use crate::log_info;
use crate::models::Message;
use crate::store::{load_vec, save_vec};

const STORE_KEY: &str = "messages";

#[tauri::command]
pub fn list_messages(app: AppHandle, session_id: String) -> Vec<Message> {
    log_info!("message", "list_messages for session {}", session_id);
    load_vec::<Message>(&app, STORE_KEY)
        .into_iter()
        .filter(|m| m.session_id == session_id)
        .collect()
}

#[tauri::command]
pub fn create_message(
    app: AppHandle,
    id: String,
    session_id: String,
    msg_type: String,
    content: String,
    created_at: String,
    thinking: Option<String>,
) -> Message {
    let metadata = thinking.map(|t| serde_json::json!({ "thinking": t }));
    let message = Message {
        id,
        session_id,
        msg_type,
        content,
        metadata,
        created_at,
    };
    let mut messages: Vec<Message> = load_vec(&app, STORE_KEY);
    messages.push(message.clone());
    save_vec(&app, STORE_KEY, &messages);
    message
}

#[tauri::command]
pub fn delete_messages(app: AppHandle, session_id: String) -> bool {
    let mut messages: Vec<Message> = load_vec(&app, STORE_KEY);
    let len_before = messages.len();
    messages.retain(|m| m.session_id != session_id);
    if messages.len() != len_before {
        save_vec(&app, STORE_KEY, &messages);
        true
    } else {
        false
    }
}
