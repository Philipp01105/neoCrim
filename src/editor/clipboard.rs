use std::sync::{Arc, Mutex};
use arboard::Clipboard as SystemClipboard;

lazy_static::lazy_static! {
    static ref INTERNAL_CLIPBOARD: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
}

pub struct Clipboard;

impl Clipboard {
    pub fn set_text(text: String) {
        if let Ok(mut clipboard) = SystemClipboard::new() {
            if let Err(_) = clipboard.set_text(&text) {
                log::warn!("Failed to set system clipboard, using internal clipboard");
                Self::set_internal_text(text);
            }
        } else {
            log::warn!("Failed to access system clipboard, using internal clipboard");
            Self::set_internal_text(text);
        }
    }

    pub fn get_text() -> String {
        if let Ok(mut clipboard) = SystemClipboard::new() {
            if let Ok(text) = clipboard.get_text() {
                return text;
            }
        }
        
        Self::get_internal_text()
    }

    pub fn clear() {
        if let Ok(mut clipboard) = SystemClipboard::new() {
            let _ = clipboard.clear();
        }
        Self::clear_internal();
    }

    fn set_internal_text(text: String) {
        if let Ok(mut clipboard) = INTERNAL_CLIPBOARD.lock() {
            *clipboard = text;
        }
    }

    fn get_internal_text() -> String {
        if let Ok(clipboard) = INTERNAL_CLIPBOARD.lock() {
            clipboard.clone()
        } else {
            String::new()
        }
    }

    fn clear_internal() {
        if let Ok(mut clipboard) = INTERNAL_CLIPBOARD.lock() {
            clipboard.clear();
        }
    }
}