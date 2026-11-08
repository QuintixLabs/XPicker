// ============================================================
// picker/mod.rs
// Picking a color: take a screenshot, open the picker, then copy,
// remember and announce the color you chose.
// ============================================================

// Windows are only ever created or closed from worker threads: doing it on the
// UI thread inside a hotkey or tray callback freezes WebView2 on Windows.

mod capture;
pub mod clipboard;
pub mod color_formats;

use std::{
    error::Error,
    sync::{atomic::{AtomicBool, Ordering}, Mutex},
    thread,
    time::Duration,
};
use tauri::{AppHandle, Manager};

use crate::{
    app::{
        settings::{self, Store},
        updates,
    },
    system::notifications,
    ui::{main_window, picker_window},
};
use clipboard::Clipboard;

/// Windows holds notification banners back while a full-screen window (the
/// picker) is in front. This gives it a moment to notice the picker is gone.
const AFTER_PICKER_CLOSES: Duration = Duration::from_millis(300);

#[derive(Default)]
pub struct Frame {
    /// Screenshot waiting to be fetched by the picker.
    pub data: Mutex<Option<Vec<u8>>>,
    /// The main window was hidden for this pick and should come back after.
    restore_main: AtomicBool,
    /// A pick is being set up; ignore repeated triggers.
    starting: AtomicBool,
    /// A pick is wrapping up (copy + notification); don't quit yet.
    finishing: AtomicBool,
    /// Quick pick: XPicker was opened just for one pick (Pick on launch), so it
    /// quits after it, even with Close to tray on.
    quick: AtomicBool,
}

impl Frame {
    pub fn is_starting(&self) -> bool {
        self.starting.load(Ordering::Acquire)
    }
}

// ------------------------------------------------------------
// Starting
// ------------------------------------------------------------

/// XPicker opened by you: picker or main window, per settings. `first_start` is
/// false when it was already running (in the tray) and you opened it again.
pub fn launch(app: &AppHandle, first_start: bool) {
    let settings = app.state::<Store>().get().settings;
    if !settings.show_welcome && settings.pick_on_launch {
        // Only a fresh start is a quick pick; one already in the tray stays there.
        if first_start {
            app.state::<Frame>().quick.store(true, Ordering::Release);
        }
        start(app);
    } else {
        main_window::open_async(app);
    }
}

/// A quick pick only lasts while it's just the picker: once you open the
/// XPicker window, it behaves like a normal start again.
pub fn end_quick_pick(app: &AppHandle) {
    app.state::<Frame>().quick.store(false, Ordering::Release);
}

/// Whether XPicker should stay in the tray when nothing is open.
pub fn stays_in_tray(app: &AppHandle) -> bool {
    app.state::<Store>().get().settings.close_to_tray && !app.state::<Frame>().quick.load(Ordering::Acquire)
}

/// Safe to call from any thread or callback.
pub fn start(app: &AppHandle) {
    let app = app.clone();
    thread::spawn(move || run(&app, false));
}

/// From the main window's button: hide it first so it isn't in the screenshot.
pub fn start_from_main(app: &AppHandle) {
    let app = app.clone();
    thread::spawn(move || run(&app, true));
}

fn run(app: &AppHandle, hide_main: bool) {
    let state = app.state::<Frame>();
    if state.starting.swap(true, Ordering::AcqRel) {
        return;
    }
    if !picker_window::is_open(app) {
        if hide_main && main_window::is_visible(app) {
            main_window::hide(app);
            state.restore_main.store(true, Ordering::Relaxed);
            thread::sleep(Duration::from_millis(180));
        }
        if let Err(err) = open(app) {
            eprintln!("XPicker: couldn't start picking: {err}");
            restore_main(app);
        }
    }
    state.starting.store(false, Ordering::Release);
}

fn open(app: &AppHandle) -> Result<(), Box<dyn Error>> {
    let cursor = app.cursor_position()?;
    let monitor = app
        .monitor_from_point(cursor.x, cursor.y)?
        .or(app.primary_monitor()?)
        .ok_or("no monitor found")?;

    let frame = capture::grab(&monitor, cursor)?;
    *app.state::<Frame>().data.lock().unwrap() = Some(frame);
    picker_window::open(app, &monitor)?;
    Ok(())
}

// ------------------------------------------------------------
// Finishing
// ------------------------------------------------------------

/// A color was picked. Runs on a worker thread (async command).
pub fn finish(app: &AppHandle, value: &str, hex: &str, format: usize) -> Result<(), String> {
    let frame = app.state::<Frame>();
    frame.finishing.store(true, Ordering::Release);

    let copied = app.state::<Clipboard>().copy(value);
    let data = app.state::<Store>().update(app, |data| {
        settings::remember(data, hex);
        data.settings.format = format;
    });

    // Close the picker first: while it's on screen, Windows would send the
    // notification straight to the notification center without showing it.
    picker_window::close(app);
    restore_main(app);
    if copied.is_ok() && data.settings.notify {
        thread::sleep(AFTER_PICKER_CLOSES);
        notifications::copied(app, value, hex);
    }

    frame.finishing.store(false, Ordering::Release);
    // Quick pick (or Close to tray off): the app stayed up only for the notification; quit now.
    if !keep_running(app) && app.webview_windows().is_empty() {
        app.exit(0);
    }
    copied
}

/// Cancel pressed in the picker.
pub fn cancel(app: &AppHandle) {
    picker_window::close(app);
    restore_main(app);
}

/// "Open XPicker" pressed in the picker: show the app, then close the picker.
pub fn open_app(app: &AppHandle) {
    let _ = main_window::open(app); // first, so there's never a moment with no window
    picker_window::close(app);
    restore_main(app);
}

/// Brings the main window back if it was hidden for this pick.
fn restore_main(app: &AppHandle) {
    if app.state::<Frame>().restore_main.swap(false, Ordering::Relaxed) {
        main_window::show(app);
    }
}

/// Asked when the last window closes. Stays alive in the tray unless "Close to
/// tray" is off or this is a quick pick; never quits halfway through a pick or
/// an update.
pub fn keep_running(app: &AppHandle) -> bool {
    let frame = app.state::<Frame>();
    frame.is_starting()
        || frame.finishing.load(Ordering::Acquire)
        || app.state::<updates::Pending>().is_busy()
        || stays_in_tray(app)
}
