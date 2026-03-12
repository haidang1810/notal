// Settings CRUD commands — persists AppSettings to/from SQLite key-value store.
// Each AppSettings field is stored as an individual key for fine-grained access.

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::db::{connection as db, DbState};
use crate::llm::provider::LLMProvider;
use crate::models::settings::{AppSettings, LlmProviderType};
use crate::services::hotkeys;
use crate::LlmState;

// ─── Key constants ────────────────────────────────────────────────────────────

const KEY_LLM_PROVIDER: &str = "llm_provider";
const KEY_OLLAMA_ENDPOINT: &str = "ollama_endpoint";
const KEY_GEMINI_API_KEY: &str = "gemini_api_key";
const KEY_GEMINI_MODEL: &str = "gemini_model";
const KEY_DECAY_WORKING: &str = "decay_rate_working";
const KEY_DECAY_EPISODIC: &str = "decay_rate_episodic";
const KEY_DECAY_SEMANTIC: &str = "decay_rate_semantic";
const KEY_CONSOLIDATION_INTERVAL: &str = "consolidation_interval_minutes";
const KEY_INBOX_FOLDER: &str = "inbox_folder_path";
const KEY_HOTKEY_CAPTURE: &str = "hotkey_capture";
const KEY_HOTKEY_OPEN: &str = "hotkey_open";
const KEY_CLIPBOARD_WATCHER: &str = "clipboard_watcher_enabled";

// ─── Commands ─────────────────────────────────────────────────────────────────

/// Reads all settings keys from SQLite and constructs an AppSettings struct.
/// Falls back to AppSettings::default() values for any missing keys.
#[tauri::command]
pub async fn get_settings(db: State<'_, DbState>) -> Result<AppSettings, String> {
    let conn = db.conn.lock().map_err(|e| format!("DB lock failed: {e}"))?;
    let defaults = AppSettings::default();

    let llm_provider = db::get_setting(&conn, KEY_LLM_PROVIDER)
        .and_then(|v| match v.as_str() {
            "ollama" => Some(LlmProviderType::Ollama),
            "gemini" => Some(LlmProviderType::Gemini),
            "auto" => Some(LlmProviderType::Auto),
            _ => None,
        })
        .unwrap_or(defaults.llm_provider);

    let ollama_endpoint = db::get_setting(&conn, KEY_OLLAMA_ENDPOINT)
        .unwrap_or(defaults.ollama_endpoint);

    let gemini_api_key = db::get_setting(&conn, KEY_GEMINI_API_KEY)
        .unwrap_or(defaults.gemini_api_key);

    let gemini_model = db::get_setting(&conn, KEY_GEMINI_MODEL)
        .unwrap_or(defaults.gemini_model);

    let decay_rate_working = db::get_setting(&conn, KEY_DECAY_WORKING)
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(defaults.decay_rate_working);

    let decay_rate_episodic = db::get_setting(&conn, KEY_DECAY_EPISODIC)
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(defaults.decay_rate_episodic);

    let decay_rate_semantic = db::get_setting(&conn, KEY_DECAY_SEMANTIC)
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(defaults.decay_rate_semantic);

    let consolidation_interval_minutes = db::get_setting(&conn, KEY_CONSOLIDATION_INTERVAL)
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(defaults.consolidation_interval_minutes);

    let inbox_folder_path = db::get_setting(&conn, KEY_INBOX_FOLDER)
        .unwrap_or(defaults.inbox_folder_path);

    let hotkey_capture = db::get_setting(&conn, KEY_HOTKEY_CAPTURE)
        .unwrap_or(defaults.hotkey_capture);

    let hotkey_open = db::get_setting(&conn, KEY_HOTKEY_OPEN)
        .unwrap_or(defaults.hotkey_open);

    let clipboard_watcher_enabled = db::get_setting(&conn, KEY_CLIPBOARD_WATCHER)
        .and_then(|v| v.parse::<bool>().ok())
        .unwrap_or(false); // off by default for privacy

    Ok(AppSettings {
        llm_provider,
        ollama_endpoint,
        gemini_api_key,
        gemini_model,
        decay_rate_working,
        decay_rate_episodic,
        decay_rate_semantic,
        consolidation_interval_minutes,
        inbox_folder_path,
        hotkey_capture,
        hotkey_open,
        clipboard_watcher_enabled,
    })
}

/// Persists all AppSettings fields as individual key-value rows in SQLite.
/// Changes take effect immediately on next worker cycle (no restart needed).
#[tauri::command]
pub async fn update_settings(
    db: State<'_, DbState>,
    settings: AppSettings,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| format!("DB lock failed: {e}"))?;

    let provider_str = match settings.llm_provider {
        LlmProviderType::Ollama => "ollama",
        LlmProviderType::Gemini => "gemini",
        LlmProviderType::Auto => "auto",
    };

    db::set_setting(&conn, KEY_LLM_PROVIDER, provider_str)
        .map_err(|e| format!("Failed to save llm_provider: {e}"))?;
    db::set_setting(&conn, KEY_OLLAMA_ENDPOINT, &settings.ollama_endpoint)
        .map_err(|e| format!("Failed to save ollama_endpoint: {e}"))?;
    db::set_setting(&conn, KEY_GEMINI_API_KEY, &settings.gemini_api_key)
        .map_err(|e| format!("Failed to save gemini_api_key: {e}"))?;
    db::set_setting(&conn, KEY_GEMINI_MODEL, &settings.gemini_model)
        .map_err(|e| format!("Failed to save gemini_model: {e}"))?;
    db::set_setting(&conn, KEY_DECAY_WORKING, &settings.decay_rate_working.to_string())
        .map_err(|e| format!("Failed to save decay_rate_working: {e}"))?;
    db::set_setting(&conn, KEY_DECAY_EPISODIC, &settings.decay_rate_episodic.to_string())
        .map_err(|e| format!("Failed to save decay_rate_episodic: {e}"))?;
    db::set_setting(&conn, KEY_DECAY_SEMANTIC, &settings.decay_rate_semantic.to_string())
        .map_err(|e| format!("Failed to save decay_rate_semantic: {e}"))?;
    db::set_setting(
        &conn,
        KEY_CONSOLIDATION_INTERVAL,
        &settings.consolidation_interval_minutes.to_string(),
    )
    .map_err(|e| format!("Failed to save consolidation_interval_minutes: {e}"))?;
    db::set_setting(&conn, KEY_INBOX_FOLDER, &settings.inbox_folder_path)
        .map_err(|e| format!("Failed to save inbox_folder_path: {e}"))?;
    db::set_setting(&conn, KEY_HOTKEY_CAPTURE, &settings.hotkey_capture)
        .map_err(|e| format!("Failed to save hotkey_capture: {e}"))?;
    db::set_setting(&conn, KEY_HOTKEY_OPEN, &settings.hotkey_open)
        .map_err(|e| format!("Failed to save hotkey_open: {e}"))?;
    db::set_setting(
        &conn,
        KEY_CLIPBOARD_WATCHER,
        &settings.clipboard_watcher_enabled.to_string(),
    )
    .map_err(|e| format!("Failed to save clipboard_watcher_enabled: {e}"))?;

    log::info!("[settings] Settings saved successfully");
    Ok(())
}

/// Re-registers global hotkeys from saved settings — call after user saves new hotkey values.
/// Unregisters all existing shortcuts first, then registers with the current DB values.
#[tauri::command]
pub async fn reregister_hotkeys(
    app: tauri::AppHandle,
    db: State<'_, DbState>,
) -> Result<(), String> {
    let settings = {
        let conn = db.conn.lock().map_err(|e| format!("DB lock: {e}"))?;
        crate::load_settings_from_db(&conn)
    };
    hotkeys::reregister_hotkeys(&app, &settings.hotkey_capture, &settings.hotkey_open)
}

/// Rebuilds LLM provider from the provided settings, replaces in-memory state, tests availability.
/// Accepts current UI settings directly so unsaved draft values are tested accurately.
#[tauri::command]
pub async fn test_llm_connection(
    llm: State<'_, LlmState>,
    settings: AppSettings,
) -> Result<bool, String> {
    let new_manager = crate::llm::create_provider(&settings)
        .map_err(|e| format!("Provider config error: {e}"))?;

    let mut guard = llm.lock().await;
    *guard = new_manager;
    Ok(guard.is_available().await)
}

// ─── Gemini model listing ───────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct GeminiModelInfo {
    pub id: String,
    pub display_name: String,
}

/// Fetches available Gemini models from the API using the provided key.
#[tauri::command]
pub async fn list_gemini_models(api_key: String) -> Result<Vec<GeminiModelInfo>, String> {
    if api_key.is_empty() {
        return Err("API key is required".into());
    }

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models?key={api_key}"
    );

    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("Request failed: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("Gemini API error: {}", resp.status()));
    }

    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Parse error: {e}"))?;

    let models = json["models"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|m| {
            let name = m["name"].as_str()?;
            let display = m["displayName"].as_str().unwrap_or(name);
            // name is "models/gemini-2.0-flash-lite" — extract the model ID
            let id = name.strip_prefix("models/").unwrap_or(name);
            // Only include generateContent-capable models
            let methods = m["supportedGenerationMethods"].as_array()?;
            let supports_generate = methods
                .iter()
                .any(|m| m.as_str() == Some("generateContent"));
            if supports_generate {
                Some(GeminiModelInfo {
                    id: id.to_string(),
                    display_name: display.to_string(),
                })
            } else {
                None
            }
        })
        .collect();

    Ok(models)
}
