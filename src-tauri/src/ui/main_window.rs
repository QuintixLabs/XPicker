// ============================================================
// main_window.rs
// The main XPicker window: opening, closing, and matching its
// title bar to the theme.
// ============================================================

use std::thread;
use tauri::{window::Color, AppHandle, Emitter, Manager, Theme, WebviewWindow, WindowEvent};

use super::{builder, logo};
use crate::app::settings::Store;

pub const LABEL: &str = "main";

// ------------------------------------------------------------
// Opening and closing
// ------------------------------------------------------------

pub fn open(app: &AppHandle) -> tauri::Result<()> {
    open_on(app, None)
}

/// For callbacks running on the UI thread (tray, single-instance).
pub fn open_async(app: &AppHandle) {
    let app = app.clone();
    thread::spawn(move || {
        let _ = open(&app);
    });
}

/// Opens the window on a page ("settings").
pub fn open_page_async(app: &AppHandle, page: &'static str) {
    let app = app.clone();
    thread::spawn(move || {
        let _ = open_on(&app, Some(page));
    });
}

/// Tray click: opens the window, or closes it if it's already open.
pub fn toggle_async(app: &AppHandle) {
    let app = app.clone();
    thread::spawn(move || match window(&app) {
        Some(window) if window.is_visible().unwrap_or(false) && !window.is_minimized().unwrap_or(false) => {
            let _ = window.close();
        }
        _ => {
            let _ = open(&app);
        }
    });
}

pub fn is_visible(app: &AppHandle) -> bool {
    window(app).is_some_and(|w| w.is_visible().unwrap_or(false))
}

/// Hidden for a pick from the window's own button, so it isn't in the screenshot.
pub fn hide(app: &AppHandle) {
    if let Some(window) = window(app) {
        let _ = window.hide();
    }
}

pub fn show(app: &AppHandle) {
    if let Some(window) = window(app) {
        let _ = window.show();
    }
}

fn window(app: &AppHandle) -> Option<WebviewWindow> {
    app.get_webview_window(LABEL)
}

fn open_on(app: &AppHandle, page: Option<&str>) -> tauri::Result<()> {
    if let Some(window) = window(app) {
        if let Some(page) = page {
            let _ = window.emit("navigate", page);
        }
        window.unminimize()?;
        window.show()?;
        return window.set_focus();
    }

    let start = page.map(|p| format!("window.__START_PAGE__ = {p:?};")).unwrap_or_default();
    let theme = window_theme(&app.state::<Store>().get().settings.theme);
    let background = if theme == Some(Theme::Light) { Color(246, 246, 247, 255) } else { Color(11, 11, 12, 255) };
    let window = builder(app, LABEL, "index.html")
        .initialization_script(&start)
        .title("XPicker")
        .inner_size(760.0, 600.0)
        .resizable(false)
        .maximizable(false)
        .center()
        .theme(theme)
        .background_color(background)
        .build()?;

    // Keep the title bar logo readable whenever the title bar changes color.
    logo::update_window(&window);
    let tracked = window.clone();
    window.on_window_event(move |event| {
        if let WindowEvent::ThemeChanged(_) = event {
            logo::update_window(&tracked);
        }
    });
    Ok(())
}

// ------------------------------------------------------------
// Theme
// ------------------------------------------------------------

/// The title bar follows XPicker's theme; None = the system's own setting.
/// On Linux the desktop draws the title bar with its own theme, so it's left alone.
fn window_theme(theme: &str) -> Option<Theme> {
    if cfg!(target_os = "linux") {
        return None;
    }
    match theme {
        "light" => Some(Theme::Light),
        "dark" => Some(Theme::Dark),
        _ => None,
    }
}

pub fn apply_theme(app: &AppHandle, theme: &str) {
    if let Some(window) = window(app) {
        let _ = window.set_theme(window_theme(theme));
        logo::update_window(&window);
    }
}
