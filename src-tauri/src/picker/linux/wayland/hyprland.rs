// ============================================================
// picker/linux/wayland/hyprland.rs
// Hyprland: where the cursor is, and the shortcut as a Hyprland
// binding, since Hyprland doesn't pass keys to other apps' shortcuts.
// ============================================================

use std::{
    io::{BufRead, BufReader, Read, Write},
    os::unix::net::UnixStream,
    path::PathBuf,
    sync::{Mutex, Once},
    thread,
    time::Duration,
};
use tauri::AppHandle;

use super::keys_of;

fn socket(name: &str) -> Option<PathBuf> {
    let signature = std::env::var("HYPRLAND_INSTANCE_SIGNATURE").ok()?;
    let runtime = std::env::var("XDG_RUNTIME_DIR").unwrap_or_default();
    [PathBuf::from(&runtime).join("hypr"), PathBuf::from("/tmp/hypr")]
        .into_iter()
        .map(|dir| dir.join(&signature).join(name))
        .find(|p| p.exists())
}

fn ask(request: &str) -> Option<String> {
    let mut stream = UnixStream::connect(socket(".socket.sock")?).ok()?;
    stream.set_read_timeout(Some(Duration::from_millis(300))).ok()?;
    stream.write_all(request.as_bytes()).ok()?;
    let mut reply = String::new();
    stream.read_to_string(&mut reply).ok()?;
    Some(reply)
}

fn ok(request: &str) -> bool {
    ask(request).is_some_and(|reply| reply.trim() == "ok")
}

/// "cursorpos" answers "1234, 567".
pub fn cursor() -> Option<(f64, f64)> {
    let reply = ask("cursorpos")?;
    let (x, y) = reply.trim().split_once(',')?;
    Some((x.trim().parse().ok()?, y.trim().parse().ok()?))
}

/// The picker has just been shown and focused. If Hyprland didn't make it
/// fullscreen, it's a box on the screen, so ask Hyprland directly.
pub fn cover_screen() {
    let own = std::process::id() as u64;
    for _ in 0..10 {
        let active = ask("j/activewindow").and_then(|reply| serde_json::from_str::<serde_json::Value>(&reply).ok());
        if let Some(window) = active.filter(|w| w["pid"].as_u64() == Some(own)) {
            let fullscreen = &window["fullscreen"];
            if !fullscreen.as_bool().unwrap_or_else(|| fullscreen.as_i64().unwrap_or(0) != 0) {
                let _ = ask("dispatch fullscreen 0");
            }
            return;
        }
        thread::sleep(Duration::from_millis(50));
    }
}

// ------------------------------------------------------------
// The shortcut
// ------------------------------------------------------------

// The binding sends XPicker an event instead of running anything. It isn't
// written to your config, so a config reload drops it; XPicker adds it again.

const EVENT: &str = "xpicker-pick";

/// The keys bound now: (modifiers, key), like (["CTRL", "SHIFT"], "X").
static BOUND: Mutex<Option<(Vec<&'static str>, String)>> = Mutex::new(None);

pub fn set_shortcut(app: &AppHandle, shortcut: &str) {
    if let Some(old) = BOUND.lock().unwrap().take() {
        unbind(&old);
    }
    let Some(keys) = keys_of(shortcut) else { return };
    if bind(&keys) {
        *BOUND.lock().unwrap() = Some(keys);
        listen(app);
    }
}

/// Up to Hyprland 0.54 binds are hyprlang keywords; from 0.55 the config is Lua.
fn bind((mods, key): &(Vec<&'static str>, String)) -> bool {
    ok(&format!("keyword bind {},{key},event,{EVENT}", mods.join(" ")))
        || ok(&format!(r#"eval hl.bind("{} + {key}", hl.dsp.event("{EVENT}"))"#, mods.join(" + ")))
}

fn unbind((mods, key): &(Vec<&'static str>, String)) {
    let _ = ok(&format!("keyword unbind {},{key}", mods.join(" ")))
        || ok(&format!(r#"eval hl.unbind("{} + {key}")"#, mods.join(" + ")));
}

/// Hyprland's events: the shortcut, and config reloads (which drop the binding).
fn listen(app: &AppHandle) {
    static STARTED: Once = Once::new();
    let app = app.clone();
    STARTED.call_once(move || {
        let _ = thread::Builder::new().name("hyprland".into()).spawn(move || loop {
            if let Some(stream) = socket(".socket2.sock").and_then(|s| UnixStream::connect(s).ok()) {
                for line in BufReader::new(stream).lines() {
                    let Ok(line) = line else { break };
                    if line == format!("custom>>{EVENT}") {
                        crate::picker::shortcut_pressed(&app);
                    } else if line.starts_with("configreloaded>>") {
                        if let Some(keys) = BOUND.lock().unwrap().as_ref() {
                            bind(keys);
                        }
                    }
                }
            }
            // Hyprland restarted or the socket went away for a moment.
            thread::sleep(Duration::from_secs(1));
        });
    });
}
