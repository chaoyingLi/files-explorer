#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use log::LevelFilter;
use simplelog::{Config, WriteLogger};
use std::fs;

fn init_logging() {
    let log_dir = dirs_next().unwrap_or_else(|| ".".into());
    fs::create_dir_all(&log_dir).ok();

    let log_file = std::path::PathBuf::from(&log_dir).join("files-explorer.log");
    let _ = WriteLogger::init(
        LevelFilter::Debug,
        Config::default(),
        fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file)
            .expect("Failed to open log file"),
    );
    log::info!("=== Files Explorer started ===");
}

fn dirs_next() -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        std::env::var("APPDATA")
            .ok()
            .map(|p| format!("{}\\files-explorer\\logs", p))
    }
    #[cfg(target_os = "macos")]
    {
        std::env::var("HOME")
            .ok()
            .map(|p| format!("{}/Library/Logs/files-explorer", p))
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        std::env::var("HOME")
            .ok()
            .map(|p| format!("{}/.local/share/files-explorer/logs", p))
    }
}

fn main() {
    init_logging();
    log::info!("Application starting");
    files_explorer_lib::run()
}
