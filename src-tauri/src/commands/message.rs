use tauri::AppHandle;

use crate::log_info;
use crate::models::Message;
use crate::store::{load_vec, save_vec};

const STORE_KEY: &str = "messages";

#[tauri::command]
pub fn list_messages(app: AppHandle, session_id: String) -> Vec<Message> {
    let all_messages: Vec<Message> = load_vec(&app, STORE_KEY);
    let filtered: Vec<Message> = all_messages
        .into_iter()
        .filter(|m| m.session_id == session_id)
        .collect();
    log_info!("message", "list_messages for session {}: found {} messages (total in store)", session_id, filtered.len());
    filtered
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
    metadata: Option<serde_json::Value>,
) -> Message {
    // Merge thinking into metadata if provided
    let final_metadata = match (thinking, metadata) {
        (Some(t), Some(mut m)) => {
            // Merge thinking into existing metadata
            if let Some(obj) = m.as_object_mut() {
                obj.insert("thinking".to_string(), serde_json::Value::String(t));
            }
            Some(m)
        }
        (Some(t), None) => {
            Some(serde_json::json!({ "thinking": t }))
        }
        (None, Some(m)) => {
            Some(m)
        }
        (None, None) => None,
    };
    let message = Message {
        id,
        session_id: session_id.clone(),
        msg_type: msg_type.clone(),
        content,
        metadata: final_metadata,
        created_at,
    };
    let mut messages: Vec<Message> = load_vec(&app, STORE_KEY);
    let total_before = messages.len();
    messages.push(message.clone());
    save_vec(&app, STORE_KEY, &messages);
    log_info!("message", "create_message: session={}, type={}, total_messages={} -> {}", session_id, msg_type, total_before, messages.len());
    message
}

#[tauri::command]
pub fn delete_messages(app: AppHandle, session_id: String) -> bool {
    let mut messages: Vec<Message> = load_vec(&app, STORE_KEY);
    let len_before = messages.len();
    messages.retain(|m| m.session_id != session_id);
    if messages.len() != len_before {
        save_vec(&app, STORE_KEY, &messages);
        log_info!("message", "delete_messages: session={}, deleted {} messages", session_id, len_before - messages.len());
        true
    } else {
        false
    }
}
