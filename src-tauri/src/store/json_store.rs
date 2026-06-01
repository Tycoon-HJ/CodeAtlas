use serde::{Deserialize, Serialize};
use tauri::AppHandle;

pub fn get_store(app: &AppHandle) -> std::sync::Arc<tauri_plugin_store::Store<tauri::Wry>> {
    tauri_plugin_store::StoreBuilder::new(app, "codeatlas.json")
        .build()
        .expect("failed to open store")
}

pub fn load_vec<T: for<'de> Deserialize<'de>>(app: &AppHandle, key: &str) -> Vec<T> {
    let store = get_store(app);
    store
        .get(key)
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default()
}

pub fn save_vec<T: Serialize>(app: &AppHandle, key: &str, data: &[T]) {
    let store = get_store(app);
    let _ = store.set(key.to_string(), serde_json::to_value(data).unwrap());
    let _ = store.save();
}

pub fn load_json<T: for<'de> Deserialize<'de>>(app: &AppHandle, key: &str) -> Option<T> {
    let store = get_store(app);
    store
        .get(key)
        .and_then(|v| serde_json::from_value(v.clone()).ok())
}

pub fn save_json<T: Serialize>(app: &AppHandle, key: &str, data: &T) {
    let store = get_store(app);
    let _ = store.set(key.to_string(), serde_json::to_value(data).unwrap());
    let _ = store.save();
}

pub fn chrono_now() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
        .to_string()
}
