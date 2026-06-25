use serde::{Deserialize, Serialize};
use std::fs;
use std::time::SystemTime;

pub fn ts_from_metadata(
    metadata: &fs::Metadata,
    getter: fn(&fs::Metadata) -> Result<SystemTime, std::io::Error>,
) -> i64 {
    getter(metadata)
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: i64,
    pub created: i64,
    pub extension: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiskInfo {
    pub name: String,
    pub mount_point: String,
    pub total_space: u64,
    pub available_space: u64,
    pub used_space: u64,
    pub file_system: String,
    pub label: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchProgress {
    pub files: Vec<FileEntry>,
    pub total: u64,
    pub done: bool,
    pub truncated: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ActionKind {
    Delete,
    Rename { old_path: String, new_path: String },
    Create { path: String, is_dir: bool },
    Copy { src: String, dest: String, was_cut: bool },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileAction {
    pub kind: ActionKind,
    pub timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClipboardInfo {
    pub paths: Vec<String>,
    pub action: String,
}

#[derive(Debug, Serialize)]
pub struct SpecialDirs {
    pub home: String,
    pub desktop: String,
    pub documents: String,
    pub downloads: String,
    pub pictures: String,
    pub music: String,
    pub videos: String,
}

// ── Constants ──
pub const LIST_BATCH_SIZE: usize = 100;
pub const PASTE_CONFLICT_SUFFIX: &str = " - Copy";
pub const SEARCH_MAX_RESULTS: u64 = 2000;
pub const SEARCH_BATCH_SIZE: usize = 500;
pub const MAX_UNDO_HISTORY: usize = 50;
