// Screenshot and quick-capture Tauri commands

use tauri::{Manager, State};
use crate::db::{DbState, connection as db};
use crate::models::note::Note;

/// Save a note originating from the quick-capture window.
/// source is set to "quick-capture" automatically.
#[tauri::command]
pub async fn save_quick_capture(
    db: State<'_, DbState>,
    raw_text: String,
) -> Result<Note, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let id = db::insert_note(&conn, &raw_text, "quick-capture")
        .map_err(|e| format!("insert_note failed: {e}"))?;
    db::get_note(&conn, id)
        .map_err(|e| format!("get_note failed: {e}"))
}

/// Save a screenshot note — stores text description with source="screenshot".
/// Actual region capture is deferred; this accepts a text description/caption.
#[tauri::command]
pub async fn save_screenshot_note(
    db: State<'_, DbState>,
    raw_text: String,
    source: Option<String>,
) -> Result<Note, String> {
    let src = source.unwrap_or_else(|| "screenshot".to_string());
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let id = db::insert_note(&conn, &raw_text, &src)
        .map_err(|e| format!("insert_note failed: {e}"))?;
    db::get_note(&conn, id)
        .map_err(|e| format!("get_note failed: {e}"))
}

/// Close the quick-capture window — called from the frontend after saving.
#[tauri::command]
pub async fn close_quick_capture(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(win) = app.get_webview_window("quick-capture") {
        win.hide().map_err(|e: tauri::Error| e.to_string())?;
    }
    Ok(())
}

/// Close the clipboard toast window — called from the frontend on save or dismiss.
#[tauri::command]
pub async fn close_clipboard_toast(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(win) = app.get_webview_window("clipboard-toast") {
        win.hide().map_err(|e: tauri::Error| e.to_string())?;
    }
    Ok(())
}

/// Close the screenshot capture window.
#[tauri::command]
pub async fn close_screenshot_capture(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(win) = app.get_webview_window("screenshot-capture") {
        win.hide().map_err(|e: tauri::Error| e.to_string())?;
    }
    Ok(())
}
