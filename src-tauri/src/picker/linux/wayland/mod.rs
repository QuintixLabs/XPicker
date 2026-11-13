// ============================================================
// picker/linux/wayland/mod.rs
// Picks the way for this desktop: Hyprland, Sway and other wlroots
// desktops capture directly; the rest go through xcap.
// ============================================================

mod hyprland;
mod wlroots;

use tauri::AppHandle;

#[derive(PartialEq)]
enum Desktop {
    Hyprland,
    Kde,
    Gnome,
    Other,
}

fn desktop() -> Desktop {
    let current = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default().to_lowercase();
    if std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_some() {
        Desktop::Hyprland
    } else if current.contains("kde") {
        Desktop::Kde
    } else if current.contains("gnome") {
        Desktop::Gnome
    } else {
        Desktop::Other
    }
}

pub fn cursor() -> Option<(f64, f64)> {
    match desktop() {
        Desktop::Hyprland => hyprland::cursor(),
        _ => None,
    }
}

pub fn capture(left: i32, top: i32, width: u32, height: u32) -> Option<Result<(u32, u32, Vec<u8>), String>> {
    match desktop() {
        Desktop::Kde | Desktop::Gnome => None,
        // Hyprland, Sway, river…: straight from the desktop, no prompt. If that
        // isn't there, xcap still tries the portal.
        _ => wlroots::capture(left, top, width, height).map(Ok),
    }
}

pub fn set_shortcut(app: &AppHandle, shortcut: &str) {
    if desktop() == Desktop::Hyprland {
        hyprland::set_shortcut(app, shortcut);
    }
}
