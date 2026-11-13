// ============================================================
// picker/linux/wayland/portal.rs
// The desktop portal: screenshots where nothing better is there.
// XPicker says who it is first, so the desktop can remember your
// answer instead of asking every time.
// ============================================================

use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{
        atomic::{AtomicU32, Ordering},
        OnceLock,
    },
};
use zbus::{
    blocking::{Connection, Proxy},
    zvariant::{OwnedValue, Value},
};

const DESKTOP: &str = "org.freedesktop.portal.Desktop";
const PATH: &str = "/org/freedesktop/portal/desktop";
/// Matches the desktop file com.quintixlabs.xpicker.desktop.
const APP_ID: &str = "com.quintixlabs.xpicker";

/// One connection for everything, introduced to the portal once. Telling it
/// who we are only works before the first portal call, and only on newer
/// portals (1.19+); older ones still work, they just ask every time.
fn connection() -> Option<&'static Connection> {
    static BUS: OnceLock<Option<Connection>> = OnceLock::new();
    BUS.get_or_init(|| {
        let bus = Connection::session().ok()?;
        let registered = bus.call_method(Some(DESKTOP), PATH, Some("org.freedesktop.host.portal.Registry"), "Register", &(APP_ID, HashMap::<&str, Value>::new()));
        debug(&format!("told the portal who XPicker is: {:?}", registered.map(|_| ())));
        Some(bus)
    })
    .as_ref()
}

/// Asks the portal something and waits for its answer. `call` makes the call,
/// given the token the answer will carry. None when you said no or it failed.
fn ask(bus: &Connection, call: impl FnOnce(&str) -> zbus::Result<zbus::message::Message>) -> Option<HashMap<String, OwnedValue>> {
    static COUNT: AtomicU32 = AtomicU32::new(0);
    let token = format!("xpicker{}", COUNT.fetch_add(1, Ordering::Relaxed));
    let sender = bus.unique_name()?.trim_start_matches(':').replace('.', "_");
    let request = Proxy::new(bus, DESKTOP, format!("{PATH}/request/{sender}/{token}"), "org.freedesktop.portal.Request").ok()?;
    // Listening before asking, so a quick answer isn't missed.
    let mut answers = request.receive_signal("Response").ok()?;
    call(&token).map_err(|e| eprintln!("XPicker: the portal said no: {e}")).ok()?;
    let (code, results): (u32, HashMap<String, OwnedValue>) = answers.next()?.body().deserialize().ok()?;
    if code != 0 {
        // 1: you said no, 2: it ended some other way.
        debug(&format!("the portal answered {code} (not 0, so no)"));
    }
    (code == 0).then_some(results)
}

/// XPICKER_DEBUG=1 prints each step, for testing.
fn debug(text: &str) {
    if std::env::var_os("XPICKER_DEBUG").is_some() {
        eprintln!("XPicker: {text}");
    }
}

// ------------------------------------------------------------
// The screenshot
// ------------------------------------------------------------

/// The portal takes the whole desktop and saves it as a file; the area is cut
/// out of it, and the file removed.
pub fn capture(left: i32, top: i32, width: u32, height: u32) -> Result<(u32, u32, Vec<u8>), String> {
    let bus = connection().ok_or("no session bus")?;
    let results = ask(bus, |token| {
        let options = HashMap::from([("handle_token", Value::from(token)), ("interactive", Value::from(false)), ("modal", Value::from(true))]);
        bus.call_method(Some(DESKTOP), PATH, Some("org.freedesktop.portal.Screenshot"), "Screenshot", &("", options))
    })
    .ok_or("the screenshot wasn't allowed")?;
    let uri = results.get("uri").and_then(|v| String::try_from(v.clone()).ok()).ok_or("the portal gave no screenshot")?;
    let file = path_of(&uri).ok_or("the portal gave no screenshot")?;
    let image = image::open(&file);
    let _ = std::fs::remove_file(&file);
    let image = image.map_err(|e| e.to_string())?.crop_imm(left.max(0) as u32, top.max(0) as u32, width, height).to_rgba8();
    Ok((image.width(), image.height(), image.into_raw()))
}

/// "file:///tmp/Screenshot%201.png" → /tmp/Screenshot 1.png
fn path_of(uri: &str) -> Option<PathBuf> {
    let encoded = uri.strip_prefix("file://")?.as_bytes();
    let mut bytes = Vec::with_capacity(encoded.len());
    let mut i = 0;
    while i < encoded.len() {
        let hex = encoded.get(i + 1..i + 3).and_then(|h| std::str::from_utf8(h).ok()).and_then(|h| u8::from_str_radix(h, 16).ok());
        match (encoded[i], hex) {
            (b'%', Some(byte)) => {
                bytes.push(byte);
                i += 3;
            }
            (byte, _) => {
                bytes.push(byte);
                i += 1;
            }
        }
    }
    Some(PathBuf::from(String::from_utf8(bytes).ok()?))
}
