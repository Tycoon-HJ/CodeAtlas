use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use once_cell::sync::Lazy;
use std::path::Path;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter};

struct WatcherState {
    watcher: Option<RecommendedWatcher>,
    watching_path: Option<String>,
}

static WATCHER_STATE: Lazy<Mutex<WatcherState>> = Lazy::new(|| {
    Mutex::new(WatcherState {
        watcher: None,
        watching_path: None,
    })
});

pub fn start_watcher(app: AppHandle, path: String) -> Result<(), String> {
    let mut state = WATCHER_STATE.lock().map_err(|e| e.to_string())?;

    // Stop existing watcher if any
    if state.watcher.is_some() {
        state.watcher = None;
        state.watching_path = None;
    }

    let watch_path = path.clone();
    let app_clone = app.clone();

    let mut watcher = RecommendedWatcher::new(
        move |result: Result<Event, notify::Error>| {
            if let Ok(event) = result {
                let kind = match &event.kind {
                    EventKind::Create(_) => "create",
                    EventKind::Modify(_) => "modify",
                    EventKind::Remove(_) => "remove",
                    _ => return,
                };

                let paths: Vec<String> = event
                    .paths
                    .iter()
                    .filter_map(|p| p.to_str().map(|s| s.to_string()))
                    .collect();

                if paths.is_empty() {
                    return;
                }

                let payload = serde_json::json!({
                    "kind": kind,
                    "paths": paths,
                });

                let _ = app_clone.emit("file-change", payload);
            }
        },
        notify::Config::default(),
    )
    .map_err(|e| format!("Failed to create watcher: {}", e))?;

    watcher
        .watch(Path::new(&watch_path), RecursiveMode::Recursive)
        .map_err(|e| format!("Failed to watch path: {}", e))?;

    state.watcher = Some(watcher);
    state.watching_path = Some(watch_path);

    Ok(())
}

pub fn stop_watcher() -> Result<(), String> {
    let mut state = WATCHER_STATE.lock().map_err(|e| e.to_string())?;
    state.watcher = None;
    state.watching_path = None;
    Ok(())
}
