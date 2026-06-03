use tauri::AppHandle;

use crate::log_info;
use crate::models::Session;
use crate::store::{chrono_now, load_vec, save_vec};

const STORE_KEY: &str = "sessions";

#[tauri::command]
pub fn list_sessions(app: AppHandle) -> Vec<Session> {
    log_info!("session", "list_sessions");
    load_vec(&app, STORE_KEY)
}

#[tauri::command]
pub fn create_session(
    app: AppHandle,
    id: String,
    task_id: String,
    provider_id: String,
    created_at: String,
    updated_at: String,
) -> Session {
    let session = Session {
        id,
        task_id,
        provider_id,
        status: "created".to_string(),
        title: None,
        token_input: 0,
        token_output: 0,
        cost: 0.0,
        created_at,
        updated_at,
    };
    let mut sessions: Vec<Session> = load_vec(&app, STORE_KEY);
    sessions.push(session.clone());
    save_vec(&app, STORE_KEY, &sessions);
    log_info!("session", "Created session {} for task {}", session.id, session.task_id);
    session
}

#[tauri::command]
pub fn update_session_status(app: AppHandle, id: String, status: String) -> bool {
    let mut sessions: Vec<Session> = load_vec(&app, STORE_KEY);
    if let Some(session) = sessions.iter_mut().find(|s| s.id == id) {
        session.status = status;
        session.updated_at = chrono_now();
        save_vec(&app, STORE_KEY, &sessions);
        true
    } else {
        false
    }
}

#[tauri::command]
pub fn delete_session(app: AppHandle, id: String) -> bool {
    let mut sessions: Vec<Session> = load_vec(&app, STORE_KEY);
    let len_before = sessions.len();
    sessions.retain(|s| s.id != id);
    if sessions.len() != len_before {
        save_vec(&app, STORE_KEY, &sessions);
        true
    } else {
        false
    }
}

#[tauri::command]
pub fn update_session_title(app: AppHandle, id: String, title: String) -> bool {
    let mut sessions: Vec<Session> = load_vec(&app, STORE_KEY);
    if let Some(session) = sessions.iter_mut().find(|s| s.id == id) {
        session.title = Some(title);
        session.updated_at = chrono_now();
        save_vec(&app, STORE_KEY, &sessions);
        true
    } else {
        false
    }
}
