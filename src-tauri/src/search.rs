use crate::types::{FileEntry, SearchProgress, ts_from_metadata, SEARCH_MAX_RESULTS, SEARCH_BATCH_SIZE};
use crate::state::AppState;
use std::fs;
use std::sync::atomic::Ordering;
use tauri::{AppHandle, Emitter, State};
use walkdir::WalkDir;

pub fn wildcard_match(pattern: &str, filename: &str) -> bool {
    let p: Vec<char> = pattern.chars().collect();
    let f: Vec<char> = filename.chars().collect();
    fn rec(p: &[char], f: &[char]) -> bool {
        match (p.is_empty(), f.is_empty()) {
            (true, true) => true,
            (true, false) => false,
            (false, true) => p.iter().all(|&c| c == '*'),
            (false, false) => {
                if p[0] == '*' {
                    rec(&p[1..], f) || rec(p, &f[1..])
                } else if p[0] == '?' || p[0].to_ascii_lowercase() == f[0].to_ascii_lowercase() {
                    rec(&p[1..], &f[1..])
                } else {
                    false
                }
            }
        }
    }
    rec(&p, &f)
}

/// Parse a size filter like ">10MB" or "<1GB" into (operator, bytes)
pub fn parse_size_filter(s: &str) -> Option<(char, u64)> {
    let s = s.trim();
    let op = s.chars().next()?;
    if op != '>' && op != '<' {
        return None;
    }
    let rest = s[1..].trim();
    // Extract numeric part
    let num_end = rest
        .find(|c: char| !c.is_ascii_digit() && c != '.')
        .unwrap_or(rest.len());
    if num_end == 0 {
        return None;
    }
    let num: f64 = rest[..num_end].parse().ok()?;
    let unit = rest[num_end..].trim().to_lowercase();
    let multiplier = match unit.as_str() {
        "b" | "" => 1.0,
        "k" | "kb" => 1024.0,
        "m" | "mb" => 1024.0 * 1024.0,
        "g" | "gb" => 1024.0 * 1024.0 * 1024.0,
        "t" | "tb" => 1024.0 * 1024.0 * 1024.0 * 1024.0,
        _ => return None,
    };
    Some((op, (num * multiplier) as u64))
}

/// Check whether a single condition matches a file
pub fn condition_matches_file(condition: &str, name: &str, size: u64) -> bool {
    let cond = condition.trim();
    if cond.is_empty() {
        return false;
    }
    // Size filter?
    if let Some((op, threshold)) = parse_size_filter(cond) {
        return match op {
            '>' => size > threshold,
            '<' => size < threshold,
            _ => false,
        };
    }
    // Wildcard pattern? (contains * or ?)
    if cond.contains('*') || cond.contains('?') {
        return wildcard_match(cond, name);
    }
    // Plain text: substring match (case-insensitive)
    name.to_lowercase().contains(&cond.to_lowercase())
}

/// Run search in a dedicated thread so it never blocks the UI
pub fn search_files(
    app: AppHandle,
    state: State<AppState>,
    directory: String,
    query: String,
    content: String,
) -> Result<(), String> {
    // Reset cancel flag
    state.search_cancel.store(false, Ordering::SeqCst);
    let cancel = state.search_cancel.clone();

    let content_clone = content.clone();
    let conditions: Vec<String> = query
        .split('|')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    if conditions.is_empty() {
        let _ = app.emit(
            "search-progress",
            SearchProgress {
                files: vec![],
                total: 0,
                done: true,
                truncated: false,
            },
        );
        return Ok(());
    }

    let dir = directory.clone();
    let app_clone = app.clone();

    // Spawn on a dedicated OS thread — never blocks Tauri's IPC thread
    std::thread::spawn(move || {
        let mut batch: Vec<FileEntry> = Vec::new();
        let mut total: u64 = 0;
        let mut truncated = false;

        let walker = WalkDir::new(&dir)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| {
                // Skip well-known directories that are never user files
                let name = e.file_name().to_str().unwrap_or("");
                !matches!(name,
                    "node_modules" | ".git" | "target" | ".next" | ".nuxt" |
                    "__pycache__" | ".venv" | "venv" | ".tox" | ".mypy_cache" |
                    "dist" | ".cache" | ".sass-cache" | ".parcel-cache" |
                    "vendor" | ".terraform" | ".serverless"
                )
            })
            .filter_map(|e| e.ok());
        for entry in walker {
            // Check cancellation (thread-safe AtomicBool)
            if cancel.load(Ordering::SeqCst) {
                let files = std::mem::take(&mut batch);
                let _ = app_clone.emit(
                    "search-progress",
                    SearchProgress {
                        files,
                        total,
                        done: true,
                        truncated: false,
                    },
                );
                return;
            }

            if total >= SEARCH_MAX_RESULTS {
                truncated = true;
                break;
            }

            let path = entry.path().to_path_buf();
            let file_name = entry.file_name().to_string_lossy().to_string();
            let metadata = match entry.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };
            let is_dir = entry.file_type().is_dir();
            let file_size = if is_dir { 0 } else { metadata.len() };
            let modified = ts_from_metadata(&metadata, fs::Metadata::modified);
            let created = ts_from_metadata(&metadata, fs::Metadata::created);
            let extension = path
                .extension()
                .map(|e| e.to_string_lossy().to_string())
                .unwrap_or_default();

            let matched = conditions
                .iter()
                .any(|cond| condition_matches_file(cond, &file_name, file_size));
            if !matched {
                continue;
            }

            // Content search (only for non-directory files)
            if !content_clone.is_empty() && !is_dir {
                // Try to read the file as text
                if let Ok(text) = std::fs::read_to_string(&path) {
                    if !text.to_lowercase().contains(&content_clone.to_lowercase()) {
                        continue;
                    }
                } else {
                    // Binary file or too large — skip
                    continue;
                }
            }

            total += 1;
            batch.push(FileEntry {
                name: file_name,
                path: path.to_string_lossy().to_string(),
                is_dir,
                size: file_size,
                modified,
                created,
                extension,
            });

            if batch.len() >= SEARCH_BATCH_SIZE {
                let files = std::mem::take(&mut batch);
                let payload = SearchProgress {
                    files,
                    total,
                    done: false,
                    truncated: false,
                };
                if app_clone.emit("search-progress", payload).is_err() {
                    break;
                }
            }
        }

        let files = std::mem::take(&mut batch);
        let _ = app_clone.emit(
            "search-progress",
            SearchProgress {
                files,
                total,
                done: true,
                truncated,
            },
        );
    });

    Ok(())
}

pub fn cancel_search(state: State<AppState>) -> Result<(), String> {
    state.search_cancel.store(true, Ordering::SeqCst);
    Ok(())
}
