#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use log::LevelFilter;
use simplelog::{Config, WriteLogger};
use std::fs;

fn init_logging() {
    let log_dir = dirs_next().unwrap_or_else(|| ".".into());
    fs::create_dir_all(&log_dir).ok();

    // Rotate: keep up to 5 log files
    let max_logs = 5;
    for i in (1..max_logs).rev() {
        let old = format!("{}/files-explorer.{}.log", log_dir, i);
        let new = format!("{}/files-explorer.{}.log", log_dir, i + 1);
        let _ = fs::remove_file(&new);
        let _ = fs::rename(&old, &new);
    }
    let main_log = format!("{}/files-explorer.log", log_dir);
    let rotated = format!("{}/files-explorer.1.log", log_dir);
    let _ = fs::rename(&main_log, &rotated);

    let log_file = std::path::PathBuf::from(&main_log);
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
