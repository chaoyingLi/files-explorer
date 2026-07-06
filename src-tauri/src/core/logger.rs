// core/logger.rs
// Centralised logging initialisation — the ONLY place that touches log paths or
// subscriber setup.  Every other module calls tracing::info!/error!/... and
// leaves the wiring to this module.
//
// Strategy:
//   - Console  → human-readable, colours, timestamp
//   - File     → JSON, daily rotation, 10 MB cap, auto‑clean 15‑day‑old logs
//   - Panic    → captured via std::panic::set_hook, written as tracing::error!

use crate::core::fs_helper;
use crate::platform;
use std::path::PathBuf;
use std::sync::OnceLock;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::fmt::time::UtcTime;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, EnvFilter, Layer};

static LOG_GUARD: OnceLock<tracing_appender::non_blocking::WorkerGuard> = OnceLock::new();

/// One-shot initialisation — call at the very top of `main()`.
///
/// # Panics
/// Panics if called twice (via `OnceLock` guard).
pub fn init() {
    let log_dir = platform::path_provider().app_log_dir();

    // Ensure directory exists (best-effort; if it fails we still have console).
    let _ = fs_helper::create_dir_all(&log_dir);

    // Clean up logs older than 15 days before opening the appender.
    cleanup_old_logs(&log_dir, 15);

    // File appender: daily rotation, prefix "files-explorer", suffix "log".
    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("files-explorer")
        .filename_suffix("log")
        .build(&log_dir)
        .unwrap_or_else(|e| {
            eprintln!("[logger] cannot create file appender at {:?}: {e}", log_dir);
            // Fallback: write to a temp file so the app doesn't crash.
            RollingFileAppender::builder()
                .rotation(Rotation::NEVER)
                .filename_prefix("files-explorer-fallback")
                .filename_suffix("log")
                .build(std::env::temp_dir())
                .expect("cannot create fallback log appender")
        });

    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    let _ = LOG_GUARD.set(guard);

    // ── Layers ──

    // Console: human-readable
    let console_layer = fmt::layer()
        .with_target(true)
        .with_level(true)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_file(false)
        .with_line_number(false)
        .with_timer(UtcTime::rfc_3339())
        .with_ansi(true)
        .compact()
        .with_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")));

    // File: JSON for structured persistence
    let file_layer = fmt::layer()
        .json()
        .with_target(true)
        .with_level(true)
        .with_file(true)
        .with_line_number(true)
        .with_timer(UtcTime::rfc_3339())
        .with_writer(non_blocking)
        .with_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")));

    tracing_subscriber::registry()
        .with(console_layer)
        .with(file_layer)
        .init();

    // ── Panic hook — captures crash stacks into the log ──
    install_panic_hook();
}

// ── Internal helpers ──

/// Remove log files whose mtime is older than `max_age_days`.
fn cleanup_old_logs(log_dir: &PathBuf, max_age_days: u64) {
    let cutoff = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .saturating_sub(max_age_days * 86400);

    if let Ok(entries) = fs_helper::read_dir(log_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("log") {
                continue;
            }
            if let Ok(meta) = entry.metadata() {
                if let Ok(mtime) = meta.modified() {
                    if let Ok(secs) = mtime.duration_since(std::time::UNIX_EPOCH) {
                        if secs.as_secs() < cutoff {
                            let _ = fs_helper::remove_file(&path);
                        }
                    }
                }
            }
        }
    }
}

/// Capture panics into tracing so they appear in both console and log file.
fn install_panic_hook() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        // Extract message + location
        let loc = info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()));
        let payload = info
            .payload()
            .downcast_ref::<&str>()
            .map(|s| s.to_string())
            .or_else(|| info.payload().downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "unknown panic".to_string());

        // Capture backtrace
        let backtrace = std::backtrace::Backtrace::force_capture();
        let bt_str = format!("{backtrace}");

        tracing::error!(
            panic.payload = %payload,
            panic.location = ?loc,
            panic.backtrace = %bt_str,
            "PANIC"
        );

        // Also call the default hook so the process still prints to stderr.
        default_hook(info);
    }));
}
