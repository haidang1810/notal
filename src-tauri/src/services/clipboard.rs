// Clipboard monitor service — polls every 1s, shows always-on-top toast window
// Privacy: disabled by default, user must explicitly enable in settings

use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, WebviewWindowBuilder};
use tauri_plugin_clipboard_manager::ClipboardExt;

/// Starts polling clipboard for text changes.
/// Only runs if `enabled` is true (disabled by default for privacy).
/// Opens a small always-on-top toast window so user sees it above all apps.
pub async fn start_clipboard_watcher(app: AppHandle, enabled: bool) {
    if !enabled {
        log::info!("[clipboard] Clipboard watcher disabled — skipping");
        return;
    }

    log::info!("[clipboard] Clipboard watcher started");
    // Seed with current clipboard content so we don't trigger on existing text at startup
    let mut last_content = app.clipboard().read_text().unwrap_or_default().trim().to_string();

    loop {
        if let Ok(current) = app.clipboard().read_text() {
            let trimmed = current.trim().to_string();
            if !trimmed.is_empty() && trimmed != last_content {
                log::debug!("[clipboard] New clipboard content detected ({} chars)", trimmed.len());
                show_clipboard_toast(&app);
                app.emit("clipboard_changed", &trimmed).ok();
                last_content = trimmed;
            }
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

/// Show or create the always-on-top clipboard toast window.
fn show_clipboard_toast(app: &AppHandle) {
    if let Some(win) = app.get_webview_window("clipboard-toast") {
        win.set_always_on_top(true).ok();
        win.show().ok();
        win.set_focus().ok();
    } else {
        let url = tauri::WebviewUrl::App("/clipboard-toast".into());
        WebviewWindowBuilder::new(app, "clipboard-toast", url)
            .title("Notal — Clipboard")
            .inner_size(360.0, 140.0)
            .always_on_top(true)
            .decorations(false)
            .resizable(false)
            .skip_taskbar(true)
            .center()
            .build()
            .ok();
    }
}
