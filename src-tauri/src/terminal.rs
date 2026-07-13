// terminal.rs
// Cross-platform PTY (pseudo-terminal) manager — multi-session.
// Zero #[cfg(target_os)] — platform shell path via platform::system_provider().
//
// Lifecycle:
//   1. Frontend invokes "terminal_spawn" → spawns shell + starts reader thread
//   2. Frontend sends keystrokes via "terminal_write" → feeds PTY stdin
//   3. Reader thread reads PTY stdout → emits "terminal-output" events (base64)
//   4. Reader thread exits → emits "terminal-exit" event
//   5. Frontend invokes "terminal_resize" → adjusts PTY dimensions
//   6. Frontend invokes "terminal_kill" → kills shell process

use crate::core::error::{AppError, AppResult};
use crate::platform;
use portable_pty::{CommandBuilder, MasterPty, PtySize};
use serde::Serialize;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::{atomic::AtomicBool, atomic::Ordering, Arc, Mutex};
use tauri::{AppHandle, Emitter};

/// Serialisable event payload sent to the frontend.
#[derive(Clone, Serialize)]
struct TermEvent {
    id: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<String>,
}

/// Internal PTY session.
struct Session {
    writer: Option<Box<dyn Write + Send>>,
    master: Option<Box<dyn MasterPty + Send>>,
    child: Option<Box<dyn portable_pty::Child + Send + Sync>>,
    cancel: Arc<AtomicBool>,
}

pub struct TerminalManager {
    sessions: Mutex<HashMap<u32, Session>>,
}

impl TerminalManager {
    pub fn new() -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
        }
    }

    /// Spawn a shell in the given directory for the given session id.
    pub fn spawn(&self, id: u32, app: AppHandle, cwd: &Path, term_type: &str) -> AppResult<()> {
        let shell = platform::system_provider().default_shell();

        if !cwd.exists() {
            return Err(AppError::NotFound(format!(
                "Directory not found: {}",
                cwd.display()
            )));
        }

        let pty_system = portable_pty::native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| AppError::IoError(format!("PTY open: {}", e)))?;

        let mut cmd = CommandBuilder::new(&shell);
        cmd.cwd(cwd);
        cmd.env("TERM", term_type);
        let locale = std::env::var("LANG").unwrap_or_else(|_| "en_US.UTF-8".into());
        cmd.env("LANG", &locale);
        cmd.env("LC_ALL", &locale);

        let child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| AppError::IoError(format!("Shell spawn: {}", e)))?;

        let writer = pair
            .master
            .take_writer()
            .map_err(|e| AppError::IoError(format!("PTY writer: {}", e)))?;

        let mut reader = pair
            .master
            .try_clone_reader()
            .map_err(|e| AppError::IoError(format!("PTY reader: {}", e)))?;

        let cancel = Arc::new(AtomicBool::new(false));

        // Kill any existing session with the same id (prevent PTY leak)
        self.kill(id);

        // Insert session
        {
            let mut guard = self
                .sessions
                .lock()
                .map_err(|e| AppError::Other(e.to_string()))?;
            guard.insert(
                id,
                Session {
                    writer: Some(writer),
                    master: Some(pair.master),
                    child: Some(child),
                    cancel: cancel.clone(),
                },
            );
        }

        // Spawn reader thread
        let app_reader = app.clone();
        std::thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                if cancel.load(Ordering::SeqCst) {
                    break;
                }
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        use base64::Engine;
                        let b64 = base64::engine::general_purpose::STANDARD.encode(&buf[..n]);
                        let _ = app_reader.emit(
                            "terminal-output",
                            TermEvent {
                                id,
                                data: Some(b64),
                            },
                        );
                    }
                    Err(_) => break,
                }
            }
            let _ = app_reader.emit("terminal-exit", TermEvent { id, data: None });
        });

        let _ = app.emit("terminal-ready", TermEvent { id, data: None });
        Ok(())
    }

    /// Write input bytes to a specific session.
    pub fn write(&self, id: u32, data: &[u8]) -> AppResult<()> {
        let mut guard = self
            .sessions
            .lock()
            .map_err(|e| AppError::Other(e.to_string()))?;
        if let Some(session) = guard.get_mut(&id) {
            if let Some(w) = session.writer.as_mut() {
                w.write_all(data)
                    .map_err(|e| AppError::IoError(format!("PTY write: {}", e)))?;
                w.flush()
                    .map_err(|e| AppError::IoError(format!("PTY flush: {}", e)))?;
            }
        }
        Ok(())
    }

    /// Resize a specific session's PTY.
    pub fn resize(&self, id: u32, rows: u16, cols: u16) -> AppResult<()> {
        let mut guard = self
            .sessions
            .lock()
            .map_err(|e| AppError::Other(e.to_string()))?;
        if let Some(session) = guard.get_mut(&id) {
            if let Some(master) = session.master.as_deref_mut() {
                master
                    .resize(PtySize {
                        rows,
                        cols,
                        pixel_width: 0,
                        pixel_height: 0,
                    })
                    .map_err(|e| AppError::IoError(format!("PTY resize: {}", e)))?;
            }
        }
        Ok(())
    }

    /// Kill a specific session.
    pub fn kill(&self, id: u32) {
        let mut guard = self.sessions.lock().ok();
        if let Some(guard) = guard.as_mut() {
            if let Some(mut session) = guard.remove(&id) {
                session.cancel.store(true, Ordering::SeqCst);
                if let Some(mut child) = session.child.take() {
                    let _ = child.kill();
                }
                session.writer = None;
                session.master = None;
            }
        }
    }

    /// Kill all sessions.
    pub fn kill_all(&self) {
        let mut guard = self.sessions.lock().ok();
        if let Some(guard) = guard.as_mut() {
            for (_, mut session) in guard.drain() {
                session.cancel.store(true, Ordering::SeqCst);
                if let Some(mut child) = session.child.take() {
                    let _ = child.kill();
                }
                session.writer = None;
                session.master = None;
            }
        }
    }
}

/// Global singleton.
use std::sync::OnceLock;
static TERMINAL: OnceLock<TerminalManager> = OnceLock::new();

pub fn terminal_state() -> &'static TerminalManager {
    TERMINAL.get_or_init(TerminalManager::new)
}
