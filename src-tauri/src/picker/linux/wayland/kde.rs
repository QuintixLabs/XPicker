// ============================================================
// picker/linux/wayland/kde.rs
// KDE Plasma: screenshots from KWin, and where the cursor is.
// ============================================================

use std::{
    collections::HashMap,
    io::Read,
    os::fd::AsFd,
    path::PathBuf,
    sync::{
        atomic::{AtomicU32, Ordering},
        mpsc::{self, Sender},
        Mutex, OnceLock,
    },
    thread,
    time::Duration,
};
use zbus::{
    blocking::Connection,
    message::Message,
    zvariant::{DynamicType, Fd, OwnedValue, Value},
};

// ------------------------------------------------------------
// The screenshot
// ------------------------------------------------------------

/// The area in the desktop's own units, at the screen's real resolution. None
/// when KWin says no.
pub fn capture(left: i32, top: i32, width: u32, height: u32) -> Option<(u32, u32, Vec<u8>)> {
    let bus = &connection()?.bus;
    let (mut reader, writer) = std::io::pipe().ok()?;
    // KWin writes the pixels while it answers, so they're read alongside.
    let pixels = thread::spawn(move || {
        let mut data = Vec::new();
        reader.read_to_end(&mut data).map(|_| data)
    });
    let options: HashMap<&str, Value> = HashMap::from([("native-resolution", Value::from(true)), ("include-cursor", Value::from(false))]);
    let reply = bus.call_method(
        Some("org.kde.KWin"),
        "/org/kde/KWin/ScreenShot2",
        Some("org.kde.KWin.ScreenShot2"),
        "CaptureArea",
        &(left, top, width, height, options, Fd::from(writer.as_fd())),
    );
    drop(writer);
    let data = pixels.join().ok()?.ok()?;
    let info: HashMap<String, OwnedValue> = reply.map_err(|e| eprintln!("XPicker: KWin didn't give a screenshot: {e}")).ok()?.body().deserialize().ok()?;
    let number = |key: &str| info.get(key).and_then(|v| u32::try_from(v).ok());
    to_rgba(&data, number("width")?, number("height")?, number("stride")?, number("format")?)
}

/// KWin sends a Qt image. Formats 4–6 are BGRA in memory, 16–18 RGBA.
fn to_rgba(data: &[u8], width: u32, height: u32, stride: u32, format: u32) -> Option<(u32, u32, Vec<u8>)> {
    let bgra = match format {
        4..=6 => true,
        16..=18 => false,
        _ => return None,
    };
    if (stride as usize) < width as usize * 4 || data.len() < stride as usize * height as usize {
        return None;
    }
    let mut out = Vec::with_capacity(width as usize * height as usize * 4);
    for row in data.chunks_exact(stride as usize).take(height as usize) {
        for px in row[..width as usize * 4].chunks_exact(4) {
            let (r, b) = if bgra { (px[2], px[0]) } else { (px[0], px[2]) };
            out.extend_from_slice(&[r, px[1], b, 255]);
        }
    }
    Some((width, height, out))
}

// ------------------------------------------------------------
// The cursor
// ------------------------------------------------------------

const SERVICE: &str = "com.quintixlabs.xpicker.Cursor";

static WAITING: Mutex<Option<Sender<(f64, f64)>>> = Mutex::new(None);

struct Reporter;

// Both come as text: KWin sends whole numbers as integers and others as
// decimals, and D-Bus turns away a call whose types don't match exactly.
#[zbus::interface(name = "com.quintixlabs.xpicker.Cursor")]
impl Reporter {
    fn report(&self, x: String, y: String) {
        if let (Ok(x), Ok(y)) = (x.trim().parse(), y.trim().parse()) {
            if let Some(waiting) = WAITING.lock().unwrap().take() {
                let _ = waiting.send((x, y));
            }
        }
    }
}

/// Runs inside KWin and calls XPicker back with the cursor. Plasma 5 and 6.
const SCRIPT: &str = r#"callDBus("__SERVICE__", "/Cursor", "__SERVICE__", "Report", String(workspace.cursorPos.x), String(workspace.cursorPos.y));"#;

struct Kwin {
    bus: Connection,
    script: PathBuf,
    count: AtomicU32,
}

/// Made once, the first time it's needed. None if the session bus isn't there.
fn connection() -> Option<&'static Kwin> {
    static KWIN: OnceLock<Option<Kwin>> = OnceLock::new();
    KWIN.get_or_init(|| {
        let bus = zbus::blocking::connection::Builder::session()
            .and_then(|b| b.name(SERVICE))
            .and_then(|b| b.serve_at("/Cursor", Reporter))
            .and_then(|b| b.build())
            .map_err(|e| eprintln!("XPicker: couldn't talk to KWin: {e}"))
            .ok()?;
        let dir = std::env::var_os("XDG_RUNTIME_DIR").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
        let script = dir.join("xpicker-cursor.js");
        std::fs::write(&script, SCRIPT.replace("__SERVICE__", SERVICE)).ok()?;
        Some(Kwin { bus, script, count: AtomicU32::new(0) })
    })
    .as_ref()
}

pub fn cursor() -> Option<(f64, f64)> {
    let kwin = connection()?;
    let (tx, rx) = mpsc::channel();
    *WAITING.lock().unwrap() = Some(tx);

    // Each run is its own small script: load, run, then unload it again.
    let name = format!("xpicker-cursor-{}", kwin.count.fetch_add(1, Ordering::Relaxed));
    let path = kwin.script.to_string_lossy().into_owned();
    let id: i32 = call(&kwin.bus, "/Scripting", "org.kde.kwin.Scripting", "loadScript", &(path, name.clone()))?.body().deserialize().ok()?;
    // Plasma 6 keeps scripts at /Scripting/Script<id>, Plasma 5 at /<id>.
    if call(&kwin.bus, &format!("/Scripting/Script{id}"), "org.kde.kwin.Script", "run", &()).is_none() {
        call(&kwin.bus, &format!("/{id}"), "org.kde.kwin.Script", "run", &());
    }
    let answer = rx.recv_timeout(Duration::from_millis(300)).ok();
    call(&kwin.bus, "/Scripting", "org.kde.kwin.Scripting", "unloadScript", &(name,));
    answer
}

fn call(bus: &Connection, path: &str, interface: &str, method: &str, args: &(impl serde::Serialize + DynamicType)) -> Option<Message> {
    bus.call_method(Some("org.kde.KWin"), path, Some(interface), method, args).ok()
}
