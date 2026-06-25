use std::path::PathBuf;
use std::sync::{Arc, Mutex, atomic::AtomicBool};
use crate::types::FileAction;

pub struct AppState {
    pub clipboard: Mutex<Vec<PathBuf>>,
    pub clipboard_action: Mutex<String>,
    pub undo_history: Mutex<Vec<FileAction>>,
    pub search_cancel: Arc<AtomicBool>,
}
