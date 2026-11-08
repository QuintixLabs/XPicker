// ============================================================
// picker_window.rs
// The full-screen picker window. A new one is made for each pick
// and closed after, so nothing stays in memory between picks.
// ============================================================

use std::sync::atomic::{AtomicUsize, Ordering};
use tauri::{AppHandle, Manager, Monitor, WebviewWindow};

use super::builder;

/// Each picker window gets a fresh label ("picker-3"), so one that's still
/// closing can never clash with the next one.
const LABEL: &str = "picker";
static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

fn all(app: &AppHandle) -> Vec<WebviewWindow> {
    let prefix = format!("{LABEL}-");
    app.webview_windows().into_iter().filter(|(label, _)| label.starts_with(&prefix)).map(|(_, w)| w).collect()
}

/// Someone is picking right now.
pub fn is_open(app: &AppHandle) -> bool {
    all(app).iter().any(|w| w.is_visible().unwrap_or(false))
}

/// Covers `monitor` with a new picker. The screenshot is already waiting; the
/// picker fetches and draws it, then shows itself (see the "ready" command).
pub fn open(app: &AppHandle, monitor: &Monitor) -> tauri::Result<()> {
    let label = format!("{LABEL}-{}", NEXT_ID.fetch_add(1, Ordering::Relaxed));
    builder(app, label, "picker.html")
        .title("XPicker")
        .decorations(false)
        .resizable(false)
        .shadow(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible(false)
        .build()
        .and_then(|window| {
            window.set_position(*monitor.position())?;
            window.set_size(*monitor.size())?;
            // Linux panels stay above normal windows (and some desktops shrink
            // windows to fit beside them), so there the picker goes fullscreen
            // to cover the whole screen, panels included.
            if cfg!(target_os = "linux") {
                window.set_fullscreen(true)?;
            }
            Ok(())
        })
}

/// After a pick or cancel.
pub fn close(app: &AppHandle) {
    for window in all(app) {
        let _ = window.destroy();
    }
}
