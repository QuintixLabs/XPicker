// Windows — the picker overlay and the main window.
// The overlay gets a fresh label each time ("overlay-3"), so one that's still
// closing can never clash with its replacement.

use std::{
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
    thread,
};
use tauri::{window::Color, AppHandle, Emitter, Manager, Monitor, Theme, WebviewUrl, WebviewWindow, WebviewWindowBuilder, WindowEvent};

use crate::{logo, store::Store};

pub const MAIN: &str = "main";
pub const OVERLAY: &str = "overlay";

static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

/// First open window of a kind ("overlay").
pub fn find(app: &AppHandle, kind: &str) -> Option<WebviewWindow> {
    let prefix = format!("{kind}-");
    app.webview_windows().into_iter().find(|(l, _)| l.starts_with(&prefix)).map(|(_, w)| w)
}

/// The window's browser data (sidebar state, theme memory) goes in an "XPicker"
/// folder too. Every window must use the same one. macOS decides this itself.
fn browser_data(app: &AppHandle) -> Option<PathBuf> {
    if cfg!(target_os = "macos") {
        return None;
    }
    app.path().local_data_dir().ok().map(|dir| dir.join("XPicker").join("WebView"))
}

fn builder<'a>(app: &'a AppHandle, label: impl Into<String>, page: &str) -> WebviewWindowBuilder<'a, tauri::Wry, AppHandle> {
    let builder = WebviewWindowBuilder::new(app, label, WebviewUrl::App(page.into()));
    match browser_data(app) {
        Some(dir) => builder.data_directory(dir),
        None => builder,
    }
}

pub fn create_overlay(app: &AppHandle, monitor: &Monitor) -> tauri::Result<()> {
    let label = format!("{OVERLAY}-{}", NEXT_ID.fetch_add(1, Ordering::Relaxed));
    let window = builder(app, label, "overlay.html")
        .title("XPicker")
        .decorations(false)
        .resizable(false)
        .shadow(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible(false)
        .build()?;

    window.set_position(*monitor.position())?;
    window.set_size(*monitor.size())?;
    Ok(())
}

/// For callbacks running on the UI thread (tray, single-instance).
pub fn open_main_async(app: &AppHandle) {
    let app = app.clone();
    thread::spawn(move || {
        let _ = open_main(&app);
    });
}

/// Tray click: opens the window, or closes it if it's already open.
pub fn toggle_main_async(app: &AppHandle) {
    let app = app.clone();
    thread::spawn(move || match app.get_webview_window(MAIN) {
        Some(window) if window.is_visible().unwrap_or(false) && !window.is_minimized().unwrap_or(false) => {
            let _ = window.close();
        }
        _ => {
            let _ = open_main(&app);
        }
    });
}

/// Opens the window on a page ("settings").
pub fn open_page_async(app: &AppHandle, page: &'static str) {
    let app = app.clone();
    thread::spawn(move || {
        let _ = open(&app, Some(page));
    });
}

pub fn open_main(app: &AppHandle) -> tauri::Result<()> {
    open(app, None)
}

fn open(app: &AppHandle, page: Option<&str>) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window(MAIN) {
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
    let window = builder(app, MAIN, "index.html")
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

/// The title bar follows XPicker's theme; None = the system's own setting.
/// On Linux the desktop draws the title bar with its own theme, so it's left alone.
pub fn window_theme(theme: &str) -> Option<Theme> {
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
    if let Some(window) = app.get_webview_window(MAIN) {
        let _ = window.set_theme(window_theme(theme));
        logo::update_window(&window);
    }
}
