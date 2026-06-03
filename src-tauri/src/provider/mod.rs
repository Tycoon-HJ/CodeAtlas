pub mod claude_adapter;
pub mod codex_adapter;

use tauri::AppHandle;

/// Common trait for all AI provider adapters.
/// Each provider (Claude, Codex, Gemini, etc.) implements this trait.
pub trait ProviderAdapter: Send + Sync {
    /// Unique provider identifier (e.g. "claude", "codex")
    fn id(&self) -> &str;

    /// Human-readable name (e.g. "Claude Code")
    fn name(&self) -> &str;

    /// Send a message to the provider and stream results via Tauri events.
    /// The adapter is responsible for emitting `claude-output` / `claude-thinking`
    /// events (or provider-specific events) to the frontend.
    fn send_message(&self, app: &AppHandle, session_id: &str, message: &str, working_dir: &str) -> Result<(), String>;

    /// Send a message with optional provider-specific config (custom binary path, config file, etc.)
    fn send_message_with_config(
        &self,
        app: &AppHandle,
        session_id: &str,
        message: &str,
        working_dir: &str,
        _provider_path: Option<&str>,
        _provider_config_path: Option<&str>,
    ) -> Result<(), String> {
        self.send_message(app, session_id, message, working_dir)
    }

    /// Stop a running session (best-effort).
    fn stop_session(&self, _session_id: &str) -> Result<(), String> {
        Ok(())
    }

    /// Whether this provider is available on the current system.
    fn is_available(&self) -> bool {
        true
    }
}

/// Registry of all available provider adapters.
pub struct ProviderRegistry {
    adapters: Vec<Box<dyn ProviderAdapter>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            adapters: Vec::new(),
        };
        // Register built-in adapters
        registry.register(Box::new(claude_adapter::ClaudeAdapter));
        registry.register(Box::new(codex_adapter::CodexAdapter));
        registry
    }

    pub fn register(&mut self, adapter: Box<dyn ProviderAdapter>) {
        self.adapters.push(adapter);
    }

    pub fn get(&self, id: &str) -> Option<&dyn ProviderAdapter> {
        self.adapters.iter().find(|a| a.id() == id).map(|a| a.as_ref())
    }

    pub fn available_providers(&self) -> Vec<(&str, &str)> {
        self.adapters
            .iter()
            .filter(|a| a.is_available())
            .map(|a| (a.id(), a.name()))
            .collect()
    }
}
