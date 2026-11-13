// ============================================================
// picker/linux/wayland/gnome/mod.rs
// GNOME: setting up and using the extension.
// ============================================================

mod shortcut;

pub use shortcut::set_shortcut;

use std::{collections::HashMap, path::PathBuf, process::Command};
use zbus::{blocking::Connection, zvariant::OwnedValue};

const UUID: &str = "xpicker@quintixlabs.com";
const SERVICE: &str = "com.quintixlabs.xpicker.Shell";
const PATH: &str = "/com/quintixlabs/xpicker/Shell";
const FILES: [(&str, &str); 2] = [("metadata.json", include_str!("extension/metadata.json")), ("extension.js", include_str!("extension/extension.js"))];

// ------------------------------------------------------------
// Using it
// ------------------------------------------------------------

fn call<R: for<'de> serde::Deserialize<'de> + zbus::zvariant::Type>(method: &str, args: &(impl serde::Serialize + zbus::zvariant::DynamicType)) -> Option<R> {
    let bus = Connection::session().ok()?;
    let reply = bus.call_method(Some(SERVICE), PATH, Some(SERVICE), method, args).ok()?;
    reply.body().deserialize().ok()
}

/// The area in GNOME's own units, at the screen's real resolution. None while
/// the extension isn't running.
pub fn capture(left: i32, top: i32, width: u32, height: u32) -> Option<(u32, u32, Vec<u8>)> {
    let dir = std::env::var_os("XDG_RUNTIME_DIR").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
    let file = dir.join("xpicker-screenshot.png");
    let taken: Option<()> = call("Screenshot", &(left, top, width as i32, height as i32, file.to_string_lossy().as_ref()));
    let image = taken.and_then(|_| image::open(&file).ok());
    let _ = std::fs::remove_file(&file);
    let image = image?.to_rgba8();
    Some((image.width(), image.height(), image.into_raw()))
}

pub fn cursor() -> Option<(f64, f64)> {
    call("Cursor", &())
}

// ------------------------------------------------------------
// Setting it up
// ------------------------------------------------------------

fn folder() -> Option<PathBuf> {
    let data = std::env::var_os("XDG_DATA_HOME").map(PathBuf::from).or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))?;
    Some(data.join("gnome-shell/extensions").join(UUID))
}

/// Puts the extension's files in place, or brings them up to date (an update
/// of XPicker can change them). GNOME reads new files at the next login.
/// Next to them goes where XPicker is: once it's gone (uninstalled), the
/// extension removes itself and the shortcut at the next login.
pub fn install() -> bool {
    let Some(dir) = folder() else { return false };
    if std::fs::create_dir_all(&dir).is_err() {
        return false;
    }
    let write = |name: &str, text: &str| {
        let path = dir.join(name);
        std::fs::read_to_string(&path).is_ok_and(|now| now == text) || std::fs::write(&path, text).is_ok()
    };
    let program = std::env::var("APPIMAGE").ok().or_else(|| std::env::current_exe().ok().map(|p| p.to_string_lossy().into_owned())).unwrap_or_default();
    FILES.iter().all(|(name, text)| write(name, text)) && write("xpicker-path", &program)
}

/// GNOME is running the extension right now.
fn running() -> bool {
    let Ok(bus) = Connection::session() else { return false };
    let Ok(reply) = bus.call_method(Some("org.gnome.Shell.Extensions"), "/org/gnome/Shell/Extensions", Some("org.gnome.Shell.Extensions"), "GetExtensionInfo", &(UUID,)) else {
        return false;
    };
    let Ok(info) = reply.body().deserialize::<HashMap<String, OwnedValue>>() else { return false };
    // 1: active (called "enabled" before GNOME 46).
    info.get("state").and_then(|state| f64::try_from(state).ok()) == Some(1.0)
}

fn gsettings(args: &[&str]) -> Option<String> {
    let out = Command::new("gsettings").args(args).output().ok()?;
    out.status.success().then(|| String::from_utf8_lossy(&out.stdout).trim().to_owned())
}

fn turned_on() -> bool {
    gsettings(&["get", "org.gnome.shell", "enabled-extensions"]).is_some_and(|list| list.contains(UUID))
}

/// For Settings: "" when there's nothing to do, "needed" before Set up, and
/// "login" once it's set up but GNOME hasn't loaded it yet.
pub fn state() -> &'static str {
    if running() {
        ""
    } else if turned_on() {
        "login"
    } else {
        "needed"
    }
}

/// The Set up button: installs the extension and turns it on.
pub fn set_up() -> Result<&'static str, String> {
    if !install() {
        return Err("Couldn't put the GNOME extension in place.".into());
    }
    if gsettings(&["get", "org.gnome.shell", "disable-user-extensions"]).as_deref() == Some("true") {
        return Err("Extensions are turned off on this computer. Turn them on in the Extensions app, then try again.".into());
    }
    let list = gsettings(&["get", "org.gnome.shell", "enabled-extensions"]).unwrap_or_default();
    if !list.contains(UUID) {
        // "['a@b', 'c@d']", or "@as []" when empty.
        let mut ids: Vec<String> = list.split('\'').skip(1).step_by(2).map(|id| format!("'{id}'")).collect();
        ids.push(format!("'{UUID}'"));
        if gsettings(&["set", "org.gnome.shell", "enabled-extensions", &format!("[{}]", ids.join(", "))]).is_none() {
            return Err("Couldn't turn on the GNOME extension.".into());
        }
    }
    // GNOME may already know it (set up before): then it works right away.
    let _ = Command::new("gnome-extensions").args(["enable", UUID]).status();
    for _ in 0..10 {
        if running() {
            return Ok("");
        }
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
    Ok("login")
}
