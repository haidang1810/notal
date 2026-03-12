// Global hotkey registration — configurable shortcuts loaded from settings

use tauri::{AppHandle, Manager, WebviewWindowBuilder};
use tauri_plugin_global_shortcut::GlobalShortcutExt;

/// Register global shortcuts using hotkey strings from settings.
/// Falls back to defaults if provided strings are empty.
pub fn setup_hotkeys(
    app: &mut tauri::App,
    hotkey_capture: &str,
    hotkey_screenshot: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let capture_key = if hotkey_capture.is_empty() {
        "Alt+Space"
    } else {
        hotkey_capture
    };

    let screenshot_key = if hotkey_screenshot.is_empty() {
        "Alt+Shift+S"
    } else {
        hotkey_screenshot
    };

    let handle = app.handle().clone();
    app.global_shortcut()
        .on_shortcut(capture_key, move |_app, _shortcut, _event| {
            toggle_quick_capture_window(&handle);
        })?;

    let handle2 = app.handle().clone();
    app.global_shortcut()
        .on_shortcut(screenshot_key, move |_app, _shortcut, _event| {
            show_screenshot_overlay(&handle2);
        })?;

    Ok(())
}

/// Re-register hotkeys at runtime — unregisters all first, then registers with new keys.
/// Called when user saves new hotkey settings.
pub fn reregister_hotkeys(
    app: &AppHandle,
    hotkey_capture: &str,
    hotkey_screenshot: &str,
) -> Result<(), String> {
    // Unregister all existing shortcuts
    app.global_shortcut()
        .unregister_all()
        .map_err(|e| format!("Failed to unregister shortcuts: {e}"))?;

    let capture_key = if hotkey_capture.is_empty() {
        "Alt+Space"
    } else {
        hotkey_capture
    };

    let screenshot_key = if hotkey_screenshot.is_empty() {
        "Alt+Shift+S"
    } else {
        hotkey_screenshot
    };

    let handle = app.clone();
    app.global_shortcut()
        .on_shortcut(capture_key, move |_app, _shortcut, _event| {
            toggle_quick_capture_window(&handle);
        })
        .map_err(|e| format!("Failed to register capture hotkey '{capture_key}': {e}"))?;

    let handle2 = app.clone();
    app.global_shortcut()
        .on_shortcut(screenshot_key, move |_app, _shortcut, _event| {
            show_screenshot_overlay(&handle2);
        })
        .map_err(|e| format!("Failed to register screenshot hotkey '{screenshot_key}': {e}"))?;

    log::info!("[hotkeys] Re-registered: capture={capture_key}, screenshot={screenshot_key}");
    Ok(())
}

/// Show quick capture window — create if absent, bring to front if exists.
/// Never hides on hotkey press — user closes via Enter (save) or Escape.
pub fn toggle_quick_capture_window(app: &AppHandle) {
    if let Some(win) = app.get_webview_window("quick-capture") {
        win.set_always_on_top(true).ok();
        win.show().ok();
        win.set_focus().ok();
    } else {
        let url = tauri::WebviewUrl::App("/quick-capture".into());
        WebviewWindowBuilder::new(app, "quick-capture", url)
            .title("Quick Capture")
            .inner_size(420.0, 140.0)
            .always_on_top(true)
            .decorations(false)
            .resizable(false)
            .center()
            .skip_taskbar(true)
            .build()
            .ok();
    }
}

/// Show screenshot capture as a floating always-on-top window.
fn show_screenshot_overlay(app: &AppHandle) {
    if let Some(win) = app.get_webview_window("screenshot-capture") {
        win.set_always_on_top(true).ok();
        win.show().ok();
        win.set_focus().ok();
    } else {
        let url = tauri::WebviewUrl::App("/screenshot-capture".into());
        WebviewWindowBuilder::new(app, "screenshot-capture", url)
            .title("Screenshot Capture")
            .inner_size(440.0, 380.0)
            .always_on_top(true)
            .decorations(false)
            .resizable(false)
            .center()
            .skip_taskbar(true)
            .build()
            .ok();
    }
}
