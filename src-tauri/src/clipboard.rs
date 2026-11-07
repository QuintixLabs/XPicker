// Clipboard — one handle kept for the app's lifetime.
// (On Linux the copied text is served from our process, so it must stay alive.)

use std::sync::Mutex;

#[derive(Default)]
pub struct Clipboard(Mutex<Option<arboard::Clipboard>>);

impl Clipboard {
    pub fn copy(&self, text: &str) -> Result<(), String> {
        let mut slot = self.0.lock().unwrap();
        if slot.is_none() {
            *slot = Some(arboard::Clipboard::new().map_err(|e| e.to_string())?);
        }
        slot.as_mut().unwrap().set_text(text).map_err(|e| e.to_string())
    }
}
