use tauri::AppHandle;

use crate::models::Workspace;
use crate::store::{load_vec, save_vec};

const STORE_KEY: &str = "workspaces";

#[tauri::command]
pub fn list_workspaces(app: AppHandle) -> Vec<Workspace> {
    load_vec(&app, STORE_KEY)
}

#[tauri::command]
pub fn create_workspace(
    app: AppHandle,
    id: String,
    name: String,
    description: Option<String>,
    created_at: String,
    updated_at: String,
) -> Workspace {
    let workspace = Workspace {
        id,
        name,
        description,
        created_at,
        updated_at,
    };
    let mut workspaces: Vec<Workspace> = load_vec(&app, STORE_KEY);
    workspaces.push(workspace.clone());
    save_vec(&app, STORE_KEY, &workspaces);
    workspace
}

#[tauri::command]
pub fn delete_workspace(app: AppHandle, id: String) -> bool {
    let mut workspaces: Vec<Workspace> = load_vec(&app, STORE_KEY);
    let len_before = workspaces.len();
    workspaces.retain(|w| w.id != id);
    if workspaces.len() != len_before {
        save_vec(&app, STORE_KEY, &workspaces);
        true
    } else {
        false
    }
}
