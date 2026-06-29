use crate::types::FileAction;
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
    /// Each call to `list_directory_streamed` increments it; spawned threads
    /// capture the generation at start and abort if it has changed, ensuring
    /// that old threads from previous navigations are cancelled without
    /// requiring a separate IPC call (avoiding race conditions).
    pub navigate_gen: Arc<AtomicU64>,
}
