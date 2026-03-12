// Inbox folder watcher — uses notify crate with debouncing for efficient file detection
// Monitors a user-configured inbox folder and auto-ingests new files as notes

use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;

use notify::{RecursiveMode, Watcher};
use notify_debouncer_full::{new_debouncer, notify::EventKind};
use tauri::{AppHandle, Emitter};

use crate::db::{connection as db, DbState};

/// File extensions supported for text ingestion as note content.
const TEXT_EXTENSIONS: &[&str] = &[
    "txt", "md", "markdown", "rst", "org",
    "json", "yaml", "yml", "toml", "csv",
    "html", "htm", "xml",
    "py", "rs", "js", "ts", "go", "rb", "sh",
    "log",
];

/// File extensions that are ingested as media notes with pending AI analysis.
const MEDIA_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "webp", "bmp",
    "pdf", "docx", "xlsx", "pptx",
    "mp3", "wav", "mp4", "mov",
];

fn is_text_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| TEXT_EXTENSIONS.contains(&e.to_lowercase().as_str()))
        .unwrap_or(false)
}

fn is_media_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| MEDIA_EXTENSIONS.contains(&e.to_lowercase().as_str()))
        .unwrap_or(false)
}

fn is_supported_file(path: &Path) -> bool {
    is_text_file(path) || is_media_file(path)
}

fn is_processed(db: &DbState, path: &Path) -> bool {
    let path_str = path.to_string_lossy().to_string();
    match db.conn.lock() {
        Ok(conn) => conn
            .query_row(
                "SELECT 1 FROM processed_files WHERE path = ?1",
                rusqlite::params![path_str],
                |_| Ok(true),
            )
            .unwrap_or(false),
        Err(_) => false,
    }
}

fn mark_processed(db: &DbState, path: &Path) {
    let path_str = path.to_string_lossy().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    if let Ok(conn) = db.conn.lock() {
        conn.execute(
            "INSERT OR IGNORE INTO processed_files (path, processed_at) VALUES (?1, ?2)",
            rusqlite::params![path_str, now],
        )
        .ok();
    }
}

async fn ingest_file_as_note(db: &DbState, path: &Path) -> Result<i64, String> {
    let file_name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let content = if is_text_file(path) {
        // Read text file content directly
        std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file '{file_name}': {e}"))?
    } else {
        // Media files get a placeholder — AI analysis queued via enrichment worker
        format!("[File: {file_name}] Pending AI analysis")
    };

    if content.trim().is_empty() {
        return Err(format!("File '{file_name}' is empty — skipping"));
    }

    let source = format!("inbox:{file_name}");
    let conn = db.conn.lock().map_err(|e| format!("DB lock failed: {e}"))?;
    let id = db::insert_note(&conn, &content, &source)
        .map_err(|e| format!("DB insert failed: {e}"))?;

    Ok(id)
}

/// Starts the inbox folder watcher using the notify crate with 500ms debouncing.
/// Creates the inbox directory if it does not exist.
/// Emits "file_ingested" events to the frontend on successful ingestion.
pub fn start_file_watcher(app: AppHandle, db: DbState, inbox_path: PathBuf) {
    // Create inbox dir if it doesn't exist
    if let Err(e) = std::fs::create_dir_all(&inbox_path) {
        log::error!("[file_watcher] Failed to create inbox dir: {e}");
        return;
    }

    log::info!("[file_watcher] Watching inbox: {}", inbox_path.display());

    let (tx, rx) = mpsc::channel();

    let mut debouncer = match new_debouncer(Duration::from_millis(500), None, tx) {
        Ok(d) => d,
        Err(e) => {
            log::error!("[file_watcher] Failed to create debouncer: {e}");
            return;
        }
    };

    if let Err(e) = debouncer.watcher().watch(&inbox_path, RecursiveMode::NonRecursive) {
        log::error!("[file_watcher] Failed to watch path: {e}");
        return;
    }

    tauri::async_runtime::spawn(async move {
        // Keep debouncer alive for the lifetime of this task
        let _debouncer = debouncer;

        while let Ok(events_result) = rx.recv() {
            match events_result {
                Ok(events) => {
                    for event in events {
                        // Only process file creation events
                        if !matches!(event.kind, EventKind::Create(_)) {
                            continue;
                        }
                        for path in &event.paths {
                            if !path.is_file() || !is_supported_file(&path) {
                                continue;
                            }
                            if is_processed(&db, &path) {
                                log::debug!(
                                    "[file_watcher] Skipping already-processed: {}",
                                    path.display()
                                );
                                continue;
                            }

                            match ingest_file_as_note(&db, &path).await {
                                Ok(note_id) => {
                                    mark_processed(&db, &path);
                                    let file_name = path
                                        .file_name()
                                        .map(|n| n.to_string_lossy().to_string())
                                        .unwrap_or_default();
                                    log::info!(
                                        "[file_watcher] Ingested '{}' as note #{note_id}",
                                        file_name
                                    );
                                    app.emit("file_ingested", &file_name).ok();
                                }
                                Err(e) => {
                                    log::error!("[file_watcher] Ingest failed: {e}");
                                }
                            }
                        }
                    }
                }
                Err(errors) => {
                    for e in errors {
                        log::error!("[file_watcher] Watch error: {e}");
                    }
                }
            }
        }

        log::warn!("[file_watcher] Watcher channel closed — exiting");
    });
}
