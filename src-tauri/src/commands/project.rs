use tauri::AppHandle;

use crate::models::Project;
use crate::store::{chrono_now, load_vec, save_vec};

const STORE_KEY: &str = "projects";

#[tauri::command]
pub fn list_projects(app: AppHandle) -> Vec<Project> {
    load_vec(&app, STORE_KEY)
}

#[tauri::command]
pub fn create_project(
    app: AppHandle,
    id: String,
    workspace_id: String,
    name: String,
    path: String,
    description: Option<String>,
    created_at: String,
    updated_at: String,
) -> Project {
    let project = Project {
        id,
        workspace_id,
        name,
        path,
        description,
        branch: None,
        last_open_time: None,
        is_favorite: None,
        created_at,
        updated_at,
    };
    let mut projects: Vec<Project> = load_vec(&app, STORE_KEY);
    projects.push(project.clone());
    save_vec(&app, STORE_KEY, &projects);
    project
}

#[tauri::command]
pub fn update_project(
    app: AppHandle,
    id: String,
    name: Option<String>,
    description: Option<String>,
    branch: Option<String>,
    is_favorite: Option<bool>,
) -> bool {
    let mut projects: Vec<Project> = load_vec(&app, STORE_KEY);
    if let Some(project) = projects.iter_mut().find(|p| p.id == id) {
        if let Some(n) = name { project.name = n; }
        if let Some(d) = description { project.description = Some(d); }
        if let Some(b) = branch { project.branch = Some(b); }
        if let Some(f) = is_favorite { project.is_favorite = Some(f); }
        project.updated_at = chrono_now();
        save_vec(&app, STORE_KEY, &projects);
        true
    } else {
        false
    }
}

#[tauri::command]
pub fn open_project(app: AppHandle, id: String) -> bool {
    let mut projects: Vec<Project> = load_vec(&app, STORE_KEY);
    if let Some(project) = projects.iter_mut().find(|p| p.id == id) {
        project.last_open_time = Some(chrono_now());
        save_vec(&app, STORE_KEY, &projects);
        true
    } else {
        false
    }
}

#[tauri::command]
pub fn delete_project(app: AppHandle, id: String) -> bool {
    let mut projects: Vec<Project> = load_vec(&app, STORE_KEY);
    let len_before = projects.len();
    projects.retain(|p| p.id != id);
    if projects.len() != len_before {
        save_vec(&app, STORE_KEY, &projects);
        true
    } else {
        false
    }
}
