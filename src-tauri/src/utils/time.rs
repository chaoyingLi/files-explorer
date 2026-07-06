// utils/time.rs
// Time helpers — platform-independent.

use std::time::Duration;

/// Run a command with a timeout, returning its output.
pub fn run_command_with_timeout(
    cmd: &str,
    args: &[&str],
    timeout: Duration,
) -> Result<std::process::Output, String> {
    let child = std::process::Command::new(cmd)
        .args(args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn {}: {}", cmd, e))?;

    let child = std::sync::Arc::new(std::sync::Mutex::new(child));
    let (tx, rx) = std::sync::mpsc::channel();
    let child_clone = child.clone();
    let handle = std::thread::spawn(move || {
        use std::io::Read;
        let mut child = child_clone.lock().unwrap();
        let status = child.wait();
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        if let Some(ref mut out) = child.stdout {
            let _ = out.read_to_end(&mut stdout);
        }
        if let Some(ref mut err) = child.stderr {
            let _ = err.read_to_end(&mut stderr);
        }
        match status {
            Ok(status) => {
                let _ = tx.send(Ok(std::process::Output {
                    status,
                    stdout,
                    stderr,
                }));
            }
            Err(e) => {
                let _ = tx.send(Err(e));
            }
        }
    });

    match rx.recv_timeout(timeout) {
        Ok(Ok(output)) => Ok(output),
        Ok(Err(e)) => Err(format!("Failed to wait for process: {}", e)),
        Err(_) => {
            let _ = child.lock().unwrap().kill();
            let _ = handle.join();
            Err(format!("Command '{}' timed out after {:?}", cmd, timeout))
        }
    }
}
