// ============================================================
// window/mod.rs
// The main window: opening, closing and its theme.
// ============================================================

pub mod commands;
pub mod logo;

use std::{path::PathBuf, thread};
use tauri::{window::Color, AppHandle, Emitter, Manager, Theme, WebviewUrl, WebviewWindow, WebviewWindowBuilder, WindowEvent};

use crate::settings::Store;

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
    // You're using XPicker now, so a quick pick no longer quits after.
    crate::picker::end_quick_pick(app);
    if let Some(window) = window(app) {
        if let Some(page) = page {
            let _ = window.emit("navigate", page);
        }
        window.unminimize()?;
        window.show()?;
        return window.set_focus();
    }

    // Told before the page draws anything, so the welcome screen never
    // shows the sidebar for a moment first.
    let settings = app.state::<Store>().get().settings;
    let page = if settings.show_welcome { Some("welcome") } else { page };
    let start = page.map(|p| format!("window.__START_PAGE__ = {p:?};")).unwrap_or_default();
    let theme = window_theme(&settings.theme);
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

    // Opened from the tray or a second start, XPicker isn't the app in front, and
    // Windows would leave a new window behind the others with only a taskbar
    // button. Asking for focus brings it to the front.
    window.set_focus()
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

// ------------------------------------------------------------
// Building it (the picker window too)
// ------------------------------------------------------------

/// The windows' browser data (sidebar state, theme memory) goes in an "XPicker"
/// folder. Every window must use the same one. macOS decides this itself.
fn browser_data(app: &AppHandle) -> Option<PathBuf> {
    if cfg!(target_os = "macos") {
        return None;
    }
    app.path().local_data_dir().ok().map(|dir| dir.join("XPicker").join("WebView"))
}

/// Starts a window for one of the pages in src/ ("index.html", "picker.html").
pub fn builder<'a>(app: &'a AppHandle, label: impl Into<String>, page: &str) -> WebviewWindowBuilder<'a, tauri::Wry, AppHandle> {
    let mut builder = WebviewWindowBuilder::new(app, label, WebviewUrl::App(page.into()));
    // On Linux the page can't always tell when the desktop switches between
    // light and dark, so it's told: here at the start, then by "system-theme".
    if cfg!(target_os = "linux") {
        builder = builder.initialization_script(format!("window.__SYSTEM_LIGHT__ = {};", logo::system_is_light()));
    }
    match browser_data(app) {
        Some(dir) => builder.data_directory(dir),
        None => builder,
    }
}
