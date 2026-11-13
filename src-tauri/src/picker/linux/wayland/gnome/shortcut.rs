// ============================================================
// picker/linux/wayland/gnome/shortcut.rs
// GNOME: the shortcut as one of GNOME's own custom shortcuts
// (Settings → Keyboard → Custom Shortcuts). GNOME runs
// "xpicker --pick", and the running XPicker opens the picker.
// ============================================================

// Apps can't hear keys pressed in other apps on Wayland, and GNOME's portal
// shortcuts need GNOME 48 and an installed desktop file. This works with any
// GNOME and any install, AppImage included.

use std::{process::Command, sync::Mutex};

const LIST: &str = "org.gnome.settings-daemon.plugins.media-keys";
const PATH: &str = "/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/xpicker/";

/// What was last written, so it isn't written again for nothing.
static WRITTEN: Mutex<Option<String>> = Mutex::new(None);

/// An empty shortcut (none, or a new key being chosen) keeps the entry but
/// leaves it without a key.
pub fn set_shortcut(shortcut: &str) {
    let binding = super::super::keys_of(shortcut).map(|(mods, key)| accelerator(&mods, &key)).unwrap_or_default();
    let mut written = WRITTEN.lock().unwrap();
    if written.as_deref() == Some(binding.as_str()) {
        return;
    }
    let entry = format!("{LIST}.custom-keybinding:{PATH}");
    let ok = add_to_list()
        && gsettings(&["set", &entry, "name", &text("XPicker: Start picking")])
        && gsettings(&["set", &entry, "command", &text(&command())])
        && gsettings(&["set", &entry, "binding", &text(&binding)]);
    if ok {
        *written = Some(binding);
    } else {
        eprintln!("XPicker: couldn't set up the GNOME shortcut");
    }
}

/// Puts XPicker's entry in GNOME's list of custom shortcuts, once.
fn add_to_list() -> bool {
    let Some(list) = read(&["get", LIST, "custom-keybindings"]) else { return false };
    if list.contains(PATH) {
        return true;
    }
    // "['/a/', '/b/']", or "@as []" when empty.
    let mut paths: Vec<&str> = list.trim_start_matches("@as").trim().trim_matches(['[', ']']).split(',').map(|p| p.trim().trim_matches('\'')).filter(|p| !p.is_empty()).collect();
    paths.push(PATH);
    let value = format!("[{}]", paths.iter().map(|p| format!("'{p}'")).collect::<Vec<_>>().join(", "));
    gsettings(&["set", LIST, "custom-keybindings", &value])
}

/// This copy of XPicker; an AppImage is run through its own file.
fn command() -> String {
    let program = std::env::var("APPIMAGE")
        .ok()
        .or_else(|| std::env::current_exe().ok().map(|p| p.to_string_lossy().into_owned()))
        .unwrap_or_else(|| "xpicker".into());
    format!("'{}' --pick", program.replace('\'', "'\\''"))
}

/// (["CTRL", "SHIFT"], "X") → "<Control><Shift>x", as GNOME writes keys.
fn accelerator(mods: &[&str], key: &str) -> String {
    let mods: String = mods
        .iter()
        .map(|m| match *m {
            "CTRL" => "<Control>",
            "SHIFT" => "<Shift>",
            "ALT" => "<Alt>",
            _ => "<Super>",
        })
        .collect();
    let key = if key.len() == 1 { key.to_lowercase() } else { key.to_string() };
    format!("{mods}{key}")
}

/// A string as gsettings reads it: 'in quotes', with \ and ' escaped.
fn text(value: &str) -> String {
    format!("'{}'", value.replace('\\', "\\\\").replace('\'', "\\'"))
}

fn gsettings(args: &[&str]) -> bool {
    Command::new("gsettings").args(args).status().is_ok_and(|s| s.success())
}

fn read(args: &[&str]) -> Option<String> {
    let out = Command::new("gsettings").args(args).output().ok()?;
    out.status.success().then(|| String::from_utf8_lossy(&out.stdout).trim().to_owned())
}
