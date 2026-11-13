// ============================================================
// picker/linux/wayland/mod.rs
// Picks the way for this desktop. Screenshots: KDE through KWin,
// GNOME through XPicker's extension, Hyprland and other wlroots desktops
// directly, and the portal for the rest. The shortcut: a Hyprland
// binding, or a GNOME custom shortcut.
// ============================================================

mod gnome;
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
        Desktop::Gnome => gnome::cursor(),
        Desktop::Other => None,
    }
}

/// The area in the desktop's own units, as (width, height, RGBA pixels).
pub fn capture(left: i32, top: i32, width: u32, height: u32) -> Result<(u32, u32, Vec<u8>), String> {
    let direct = match desktop() {
        // No prompt for the .deb and .rpm (their desktop file allows it).
        Desktop::Kde => kde::capture(left, top, width, height),
        // No prompt and no camera sound once the extension is set up.
        Desktop::Gnome => gnome::capture(left, top, width, height),
        // Hyprland, Sway, river…: no prompt.
        _ => wlroots::capture(left, top, width, height),
    };
    match direct {
        Some(shot) => Ok(shot),
        None => portal::capture(left, top, width, height),
    }
}

/// When XPicker starts: GNOME's extension files are kept up to date.
pub fn start() {
    if desktop() == Desktop::Gnome {
        gnome::install();
    }
}

/// For Settings: what's left to set up on this desktop ("" for nothing).
pub fn setup_state() -> &'static str {
    if desktop() == Desktop::Gnome { gnome::state() } else { "" }
}

pub fn set_up() -> Result<&'static str, String> {
    if desktop() == Desktop::Gnome { gnome::set_up() } else { Ok("") }
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
        Desktop::Gnome => gnome::set_shortcut(shortcut),
        // KDE passes keys to Tauri's own shortcut, so it needs nothing.
        Desktop::Kde | Desktop::Other => {}
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
