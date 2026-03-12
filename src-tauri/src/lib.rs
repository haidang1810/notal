// Notal — Tauri v2 backend entry point

use std::path::PathBuf;
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::Mutex;

pub mod commands;
pub mod db;
pub mod llm;
pub mod memory;
pub mod models;
pub mod services;

/// Shared LLM state — Arc<Mutex<>> so commands can swap providers at runtime.
pub type LlmState = Arc<Mutex<llm::FallbackManager>>;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::notes::create_note,
            commands::notes::get_notes,
            commands::notes::get_note_by_id,
            commands::notes::update_note,
            commands::notes::delete_note,
            commands::notes::ingest_file,
            commands::search::search_notes,
            commands::search::ask_ai,
            commands::memory::get_memory_stats,
            commands::consolidation::trigger_consolidation,
            commands::capture::save_quick_capture,
            commands::capture::save_screenshot_note,
            commands::capture::close_quick_capture,
            commands::capture::close_clipboard_toast,
            commands::capture::close_screenshot_capture,
            commands::settings::get_settings,
            commands::settings::update_settings,
            commands::settings::test_llm_connection,
            commands::settings::reregister_hotkeys,
            commands::settings::list_gemini_models,
        ])
        .setup(|app| {
            // Initialise SQLite database in the OS-protected app data directory
            let app_data_dir = app.path().app_data_dir().expect("no app data dir");
            std::fs::create_dir_all(&app_data_dir).ok();
            let db_path = app_data_dir.join("notal.db");
            let db_state = db::connection::init_db(&db_path)
                .map_err(|e| Box::<dyn std::error::Error>::from(e))?;
            app.manage(db_state);

            // Load persisted settings (or use defaults for first launch)
            let settings = {
                let db = app.state::<db::DbState>();
                let conn = db.conn.lock().expect("DB lock");
                load_settings_from_db(&conn)
            };

            // Initialise LLM provider from loaded settings.
            // Failures are non-fatal — app runs in offline mode until provider is configured.
            match llm::create_provider(&settings) {
                Ok(manager) => {
                    let llm_state: LlmState = Arc::new(Mutex::new(manager));
                    app.manage(llm_state);
                }
                Err(e) => {
                    log::warn!("LLM provider init failed (offline mode): {e}");
                }
            }

            // Spawn background enrichment worker (needs both states to be managed first)
            let db_clone = app.state::<db::DbState>().inner().clone();
            if let Some(llm_state) = app.try_state::<LlmState>() {
                let llm_clone = llm_state.inner().clone();
                let handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    services::enrichment::start_enrichment_worker(db_clone, llm_clone, handle)
                        .await;
                });

                // Spawn background consolidation worker
                let db_clone2 = app.state::<db::DbState>().inner().clone();
                let llm_clone2 = llm_state.inner().clone();
                let handle2 = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    memory::start_consolidation_worker(
                        db_clone2,
                        llm_clone2,
                        handle2,
                        memory::ConsolidationConfig::default(),
                    )
                    .await;
                });
            } else {
                log::info!("Enrichment worker not started — LLM unavailable (offline mode)");
            }

            // Spawn clipboard watcher if enabled in settings (off by default for privacy)
            if settings.clipboard_watcher_enabled {
                let handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    services::clipboard::start_clipboard_watcher(handle, true).await;
                });
            } else {
                log::info!("[clipboard] Clipboard watcher disabled in settings");
            }

            // Resolve inbox folder path — use configured value or default to ~/Notal/inbox
            let inbox_path = resolve_inbox_path(&settings.inbox_folder_path);

            // Spawn file watcher for inbox folder
            let db_for_watcher = app.state::<db::DbState>().inner().clone();
            let handle_for_watcher = app.handle().clone();
            services::file_watcher::start_file_watcher(
                handle_for_watcher,
                db_for_watcher,
                inbox_path,
            );

            // Register global hotkeys using settings values (or defaults if empty)
            services::hotkeys::setup_hotkeys(app, &settings.hotkey_capture, &settings.hotkey_open)
                .map_err(|e| Box::<dyn std::error::Error>::from(e.to_string()))?;

            // Build system tray with menu
            services::tray::setup_tray(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            // Hide main window on close instead of quitting — keeps app in tray
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    window.hide().ok();
                    api.prevent_close();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Loads AppSettings from the DB connection; returns defaults for any missing keys.
pub fn load_settings_from_db(conn: &rusqlite::Connection) -> models::settings::AppSettings {
    use crate::db::connection as db;
    use crate::models::settings::{AppSettings, LlmProviderType};

    let defaults = AppSettings::default();

    let llm_provider = db::get_setting(conn, "llm_provider")
        .and_then(|v| match v.as_str() {
            "ollama" => Some(LlmProviderType::Ollama),
            "gemini" => Some(LlmProviderType::Gemini),
            "auto"   => Some(LlmProviderType::Auto),
            _ => None,
        })
        .unwrap_or(defaults.llm_provider);

    AppSettings {
        llm_provider,
        ollama_endpoint: db::get_setting(conn, "ollama_endpoint")
            .unwrap_or(defaults.ollama_endpoint),
        gemini_api_key: db::get_setting(conn, "gemini_api_key")
            .unwrap_or(defaults.gemini_api_key),
        gemini_model: db::get_setting(conn, "gemini_model")
            .unwrap_or(defaults.gemini_model),
        decay_rate_working: db::get_setting(conn, "decay_rate_working")
            .and_then(|v| v.parse().ok())
            .unwrap_or(defaults.decay_rate_working),
        decay_rate_episodic: db::get_setting(conn, "decay_rate_episodic")
            .and_then(|v| v.parse().ok())
            .unwrap_or(defaults.decay_rate_episodic),
        decay_rate_semantic: db::get_setting(conn, "decay_rate_semantic")
            .and_then(|v| v.parse().ok())
            .unwrap_or(defaults.decay_rate_semantic),
        consolidation_interval_minutes: db::get_setting(conn, "consolidation_interval_minutes")
            .and_then(|v| v.parse().ok())
            .unwrap_or(defaults.consolidation_interval_minutes),
        inbox_folder_path: db::get_setting(conn, "inbox_folder_path")
            .unwrap_or(defaults.inbox_folder_path),
        hotkey_capture: db::get_setting(conn, "hotkey_capture")
            .unwrap_or(defaults.hotkey_capture),
        hotkey_open: db::get_setting(conn, "hotkey_open")
            .unwrap_or(defaults.hotkey_open),
        clipboard_watcher_enabled: db::get_setting(conn, "clipboard_watcher_enabled")
            .and_then(|v| v.parse().ok())
            .unwrap_or(false),
    }
}

/// Resolves the inbox folder path.
/// If the configured path is empty, defaults to ~/Notal/inbox.
fn resolve_inbox_path(configured: &str) -> PathBuf {
    if !configured.is_empty() {
        return PathBuf::from(configured);
    }
    // Default: ~/Notal/inbox
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Notal")
        .join("inbox")
}
