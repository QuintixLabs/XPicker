// ============================================================
// picker/linux/wayland/mod.rs
// Picks the way for this desktop. Screenshots: KDE through KWin,
// Hyprland and other wlroots desktops directly, and the portal for the
// rest. The shortcut: a Hyprland binding, or the portal on GNOME.
// ============================================================

mod hyprland;
mod kde;
mod portal;
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
        Desktop::Kde => kde::cursor(),
        _ => None,
    }
}

/// The area in the desktop's own units, as (width, height, RGBA pixels).
pub fn capture(left: i32, top: i32, width: u32, height: u32) -> Result<(u32, u32, Vec<u8>), String> {
    let direct = match desktop() {
        // No prompt for the .deb and .rpm (their desktop file allows it).
        Desktop::Kde => kde::capture(left, top, width, height),
        Desktop::Gnome => None,
        // Hyprland, Sway, river…: no prompt.
        _ => wlroots::capture(left, top, width, height),
    };
    match direct {
        Some(shot) => Ok(shot),
        None => portal::capture(left, top, width, height),
    }
}

/// The picker was just shown: make sure it covers the whole screen.
pub fn cover_screen() {
    if desktop() == Desktop::Hyprland {
        hyprland::cover_screen();
    }
}

pub fn set_shortcut(app: &AppHandle, shortcut: &str) {
    match desktop() {
        Desktop::Hyprland => hyprland::set_shortcut(app, shortcut),
        // KDE passes keys to Tauri's own shortcut, so it needs nothing.
        Desktop::Gnome | Desktop::Other => portal::set_shortcut(app, shortcut),
        Desktop::Kde => {}
    }
}

// ------------------------------------------------------------
// Key names
// ------------------------------------------------------------

/// "Ctrl+Shift+KeyX" → (["CTRL", "SHIFT"], "X"). None for keys that can't be
/// named outside the page (media keys and such).
fn keys_of(shortcut: &str) -> Option<(Vec<&'static str>, String)> {
    let (mut mods, mut key) = (Vec::new(), None);
    for part in shortcut.split('+') {
        match part {
            "Ctrl" | "Control" => mods.push("CTRL"),
            "Shift" => mods.push("SHIFT"),
            "Alt" => mods.push("ALT"),
            "Super" | "Meta" | "Cmd" => mods.push("SUPER"),
            code => key = Some(key_name(code)?),
        }
    }
    Some((mods, key?))
}

/// A key code from the page ("KeyX", "Digit1", "F5") as Linux names it.
fn key_name(code: &str) -> Option<String> {
    if let Some(rest) = code.strip_prefix("Key").or_else(|| code.strip_prefix("Digit")) {
        return Some(rest.into());
    }
    if let Some(rest) = code.strip_prefix("Numpad").filter(|r| r.len() == 1) {
        return Some(format!("KP_{rest}"));
    }
    if code.len() > 1 && code.starts_with('F') && code[1..].chars().all(|c| c.is_ascii_digit()) {
        return Some(code.into());
    }
    let name = match code {
        "Space" => "space",
        "Enter" => "Return",
        "Escape" => "Escape",
        "Tab" => "Tab",
        "Backspace" => "BackSpace",
        "Delete" => "Delete",
        "Insert" => "Insert",
        "Home" => "Home",
        "End" => "End",
        "PageUp" => "Prior",
        "PageDown" => "Next",
        "ArrowUp" => "Up",
        "ArrowDown" => "Down",
        "ArrowLeft" => "Left",
        "ArrowRight" => "Right",
        "Minus" => "minus",
        "Equal" => "equal",
        "Comma" => "comma",
        "Period" => "period",
        "Slash" => "slash",
        "Backslash" => "backslash",
        "Semicolon" => "semicolon",
        "Quote" => "apostrophe",
        "Backquote" => "grave",
        "BracketLeft" => "bracketleft",
        "BracketRight" => "bracketright",
        _ => return None,
    };
    Some(name.into())
}
