use std::sync::{Arc, Mutex};

lazy_static::lazy_static! {
    static ref CLIPBOARD: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
}

pub struct Clipboard;

impl Clipboard {
    pub fn set_text(text: String) {
        if let Ok(mut clipboard) = CLIPBOARD.lock() {
            *clipboard = text;
        }
    }

    pub fn get_text() -> String {
        if let Ok(clipboard) = CLIPBOARD.lock() {
            clipboard.clone()
        } else {
            String::new()
        }
    }

    pub fn clear() {
        if let Ok(mut clipboard) = CLIPBOARD.lock() {
            clipboard.clear();
        }
    }
}