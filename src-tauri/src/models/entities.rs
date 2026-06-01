use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Project {
    pub id: String,
    #[serde(rename = "workspaceId")]
    pub workspace_id: String,
    pub name: String,
    pub path: String,
    pub description: Option<String>,
    pub branch: Option<String>,
    #[serde(rename = "lastOpenTime")]
    pub last_open_time: Option<String>,
    #[serde(rename = "isFavorite")]
    pub is_favorite: Option<bool>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Session {
    pub id: String,
    #[serde(rename = "taskId")]
    pub task_id: String,
    #[serde(rename = "providerId")]
    pub provider_id: String,
    pub status: String,
    pub title: Option<String>,
    #[serde(rename = "tokenInput")]
    pub token_input: u64,
    #[serde(rename = "tokenOutput")]
    pub token_output: u64,
    pub cost: f64,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Message {
    pub id: String,
    #[serde(rename = "sessionId")]
    pub session_id: String,
    #[serde(rename = "type")]
    pub msg_type: String,
    pub content: String,
    pub metadata: Option<serde_json::Value>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

#[derive(Serialize, Clone)]
pub struct ClaudeOutput {
    pub session_id: String,
    pub content: String,
    pub thinking: String,
    pub done: bool,
}
