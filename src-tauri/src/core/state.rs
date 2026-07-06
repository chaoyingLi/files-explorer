// core/state.rs
// Application-wide shared state.

use crate::core::types::FileAction;
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, AtomicU64},
    Arc, Mutex,
};

pub struct AppStateInner {
    pub clipboard: Vec<PathBuf>,
    pub clipboard_action: String,
    pub undo_history: Vec<FileAction>,
}

pub struct AppState {
    pub inner: Mutex<AppStateInner>,
    pub search_cancel: Arc<AtomicBool>,
    /// Monotonically increasing generation counter for directory listings.
    pub navigate_gen: Arc<AtomicU64>,
    /// Whether to close window instead of hiding to tray.
    pub quit_on_close: Arc<AtomicBool>,
}
