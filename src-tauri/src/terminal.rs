// terminal.rs
// Cross-platform PTY (pseudo-terminal) manager.
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
use std::io::{Read, Write};
use std::path::Path;
use std::sync::{atomic::AtomicBool, atomic::Ordering, Arc, Mutex};
use tauri::{AppHandle, Emitter};

/// Internal PTY session.
struct Inner {
    writer: Option<Box<dyn Write + Send>>,
    master: Option<Box<dyn MasterPty + Send>>,
    child: Option<Box<dyn portable_pty::Child + Send + Sync>>,
    cancel: Arc<AtomicBool>,
}

pub struct TerminalState {
    inner: Mutex<Inner>,
}

impl TerminalState {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(Inner {
                writer: None,
                master: None,
                child: None,
                cancel: Arc::new(AtomicBool::new(false)),
            }),
        }
    }

    /// Spawn a shell in the given directory.
    pub fn spawn(&self, app: AppHandle, cwd: &Path) -> AppResult<()> {
        // Kill any existing session
        {
            let mut guard = self
                .inner
                .lock()
                .map_err(|e| AppError::Other(e.to_string()))?;
            if guard.child.is_some() {
                guard.cancel.store(true, Ordering::SeqCst);
                let _ = guard.child.take();
            }
            guard.writer = None;
            guard.master = None;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));

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

        let mut guard = self
            .inner
            .lock()
            .map_err(|e| AppError::Other(e.to_string()))?;
        guard.writer = Some(writer);
        guard.master = Some(pair.master);
        guard.child = Some(child);
        guard.cancel = cancel.clone();
        drop(guard);

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
                        let _ = app_reader.emit("terminal-output", b64);
                    }
                    Err(_) => break,
                }
            }
            let _ = app_reader.emit("terminal-exit", ());
        });

        let _ = app.emit("terminal-ready", ());
        Ok(())
    }

    /// Write input bytes to the PTY.
    pub fn write(&self, data: &[u8]) -> AppResult<()> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|e| AppError::Other(e.to_string()))?;
        if let Some(w) = guard.writer.as_mut() {
            w.write_all(data)
                .map_err(|e| AppError::IoError(format!("PTY write: {}", e)))?;
            w.flush()
                .map_err(|e| AppError::IoError(format!("PTY flush: {}", e)))?;
        }
        Ok(())
    }

    /// Resize the PTY.
    pub fn resize(&self, rows: u16, cols: u16) -> AppResult<()> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|e| AppError::Other(e.to_string()))?;
        if let Some(master) = guard.master.as_deref_mut() {
            master
                .resize(PtySize {
                    rows,
                    cols,
                    pixel_width: 0,
                    pixel_height: 0,
                })
                .map_err(|e| AppError::IoError(format!("PTY resize: {}", e)))?;
        }
        Ok(())
    }

    /// Kill the child process and clean up.
    pub fn kill(&self) {
        let mut guard = self.inner.lock().ok();
        if let Some(guard) = guard.as_mut() {
            guard.cancel.store(true, Ordering::SeqCst);
            if let Some(mut child) = guard.child.take() {
                let _ = child.kill();
            }
            guard.writer = None;
            guard.master = None;
        }
    }
}

/// Global singleton.
use std::sync::OnceLock;
static TERMINAL: OnceLock<TerminalState> = OnceLock::new();

pub fn terminal_state() -> &'static TerminalState {
    TERMINAL.get_or_init(TerminalState::new)
}
