use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LlmProviderType {
    Ollama,
    Gemini,
    Auto,
}

impl Default for LlmProviderType {
    fn default() -> Self {
        Self::Auto
    }
}

/// Application settings stored in the settings key-value table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub llm_provider: LlmProviderType,
    pub ollama_endpoint: String,
    pub gemini_api_key: String,
    /// Gemini model name (e.g. "gemini-2.0-flash-lite")
    pub gemini_model: String,
    /// Decay rate applied per hour for the working memory layer
    pub decay_rate_working: f64,
    /// Decay rate applied per hour for the episodic memory layer
    pub decay_rate_episodic: f64,
    /// Decay rate applied per hour for the semantic memory layer
    pub decay_rate_semantic: f64,
    /// How often (in minutes) to run memory consolidation
    pub consolidation_interval_minutes: u64,
    /// Folder path monitored by the file inbox watcher
    pub inbox_folder_path: String,
    /// Global hotkey to open quick-capture window
    pub hotkey_capture: String,
    /// Global hotkey to open main window
    pub hotkey_open: String,
    /// Whether to monitor clipboard for text changes (off by default for privacy)
    pub clipboard_watcher_enabled: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            llm_provider: LlmProviderType::Auto,
            ollama_endpoint: "http://localhost:11434".to_string(),
            gemini_api_key: String::new(),
            gemini_model: "gemini-2.0-flash-lite".to_string(),
            decay_rate_working: 0.1,
            decay_rate_episodic: 0.05,
            decay_rate_semantic: 0.01,
            consolidation_interval_minutes: 60,
            inbox_folder_path: String::new(),
            hotkey_capture: "Ctrl+Shift+N".to_string(),
            hotkey_open: "Ctrl+Shift+S".to_string(),
            clipboard_watcher_enabled: false,
        }
    }
}
