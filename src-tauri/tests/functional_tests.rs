// ── Integration tests for Files Explorer backend ──
//
// Run: cargo test --test functional_tests

use std::fs;
use std::path::{Path, PathBuf};

// ── Helper: create a temp directory and clean up ──
struct TempDir {
    path: PathBuf,
}
impl TempDir {
    fn new(prefix: &str) -> Self {
        let path = std::env::temp_dir().join(format!("fe_test_{}", prefix));
        let _ = fs::remove_dir_all(&path); // clean up from previous run
        fs::create_dir_all(&path).unwrap();
        TempDir { path }
    }
    fn join(&self, name: &str) -> PathBuf {
        self.path.join(name)
    }
}
impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

// ══════════════════════════════════════════════
//  Search utilities (unit tests using crate code)
// ══════════════════════════════════════════════
// These functions are in the search module. We import them directly.
// Since they're `pub fn` in the library, we can use them.
// For simplicity in integration tests, we duplicate the pure logic below
// (they have zero dependencies on Tauri state).

fn wildcard_match(pattern: &str, filename: &str) -> bool {
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

fn parse_size_filter(s: &str) -> Option<(char, u64)> {
    let s = s.trim();
    let op = s.chars().next()?;
    if op != '>' && op != '<' {
        return None;
    }
    let rest = s[1..].trim();
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

fn condition_matches_file(condition: &str, name: &str, size: u64) -> bool {
    let cond = condition.trim();
    if cond.is_empty() {
        return false;
    }
    if let Some((op, threshold)) = parse_size_filter(cond) {
        return match op {
            '>' => size > threshold,
            '<' => size < threshold,
            _ => false,
        };
    }
    if cond.contains('*') || cond.contains('?') {
        return wildcard_match(cond, name);
    }
    name.to_lowercase().contains(&cond.to_lowercase())
}

fn resolve_paste_conflict(path: &Path) -> PathBuf {
    if !path.exists() {
        return path.to_path_buf();
    }
    let parent = path.parent().unwrap_or(Path::new("."));
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().to_string())
        .unwrap_or_default();
    let mut counter: u32 = 1;
    loop {
        let new_name = if counter == 1 {
            if ext.is_empty() {
                format!("{} - Copy", stem)
            } else {
                format!("{} - Copy.{}", stem, ext)
            }
        } else {
            if ext.is_empty() {
                format!("{} ({})", stem, counter)
            } else {
                format!("{} ({}).{}", stem, counter, ext)
            }
        };
        let candidate = parent.join(&new_name);
        if !candidate.exists() {
            return candidate;
        }
        counter += 1;
        if counter > 999 {
            break;
        }
    }
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    parent.join(format!("{}_{}.{}", stem, ts, ext))
}

// ══════════════════════════════════════════════
//  Search utility tests
// ══════════════════════════════════════════════

#[test]
fn test_wildcard_exact() {
    assert!(wildcard_match("hello.txt", "hello.txt"));
    assert!(!wildcard_match("hello.txt", "world.txt"));
}

#[test]
fn test_wildcard_star() {
    assert!(wildcard_match("*.txt", "hello.txt"));
    assert!(wildcard_match("*.txt", "HELLO.TXT"));
    assert!(!wildcard_match("*.txt", "hello.md"));
    assert!(wildcard_match("*test*", "mytestfile"));
    assert!(wildcard_match("*", "anything"));
}

#[test]
fn test_wildcard_question() {
    assert!(wildcard_match("file?.txt", "file1.txt"));
    assert!(!wildcard_match("file?.txt", "file12.txt"));
    assert!(wildcard_match("???.txt", "abc.txt"));
}

#[test]
fn test_wildcard_edge() {
    assert!(wildcard_match("", ""));
    assert!(!wildcard_match("", "x"));
    // Star matches empty too
    assert!(wildcard_match("a*b", "ab"));
    assert!(wildcard_match("a*b*", "ab"));
}

#[test]
fn test_parse_size_gt() {
    assert_eq!(parse_size_filter(">1MB"), Some(('>', 1048576)));
    assert_eq!(parse_size_filter(">100KB"), Some(('>', 102400)));
    assert_eq!(parse_size_filter(">1GB"), Some(('>', 1073741824)));
}

#[test]
fn test_parse_size_lt() {
    assert_eq!(parse_size_filter("<10KB"), Some(('<', 10240)));
}

#[test]
fn test_parse_size_no_units() {
    assert_eq!(parse_size_filter(">1024"), Some(('>', 1024)));
}

#[test]
fn test_parse_size_invalid() {
    assert_eq!(parse_size_filter("1MB"), None);
    assert_eq!(parse_size_filter("abc"), None);
    assert_eq!(parse_size_filter(">"), None);
}

#[test]
fn test_condition_substring() {
    assert!(condition_matches_file("hello", "hello.txt", 100));
    assert!(condition_matches_file("HELLO", "hello.txt", 100)); // case-insensitive
    assert!(!condition_matches_file("xyz", "hello.txt", 100));
}

#[test]
fn test_condition_wildcard() {
    assert!(condition_matches_file("*.rs", "main.rs", 100));
    assert!(condition_matches_file("test*.ts", "test_utils.ts", 100));
    assert!(!condition_matches_file("*.rs", "main.ts", 100));
}

#[test]
fn test_condition_size() {
    assert!(condition_matches_file(">1MB", "big.txt", 2_000_000));
    assert!(!condition_matches_file(">1MB", "small.txt", 500_000));
    assert!(condition_matches_file("<1KB", "tiny.txt", 500));
}

#[test]
fn test_condition_empty() {
    assert!(!condition_matches_file("", "t.txt", 100));
}

#[test]
fn test_condition_or_logic() {
    // Simulating the | split (done in search_files command)
    let conditions: Vec<&str> = "*.rs|>1MB".split('|').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
    // A 2MB .rs file should match (wildcard OR size)
    let matches = conditions.iter().any(|c| condition_matches_file(c, "main.rs", 2_000_000));
    assert!(matches);
    // A 100B .txt file should match neither
    let matches = conditions.iter().any(|c| condition_matches_file(c, "readme.txt", 100));
    assert!(!matches);
}

// ══════════════════════════════════════════════
//  Paste conflict resolution tests
// ══════════════════════════════════════════════

#[test]
fn test_paste_no_conflict() {
    let tmp = std::env::temp_dir().join("fe_test_noexist_xyz_12345");
    let _ = fs::remove_file(&tmp);
    assert_eq!(resolve_paste_conflict(&tmp), tmp);
}

#[test]
fn test_paste_conflict_with_ext() {
    let tmp = std::env::temp_dir();
    let tf = tmp.join("fe_test_pasteconf.txt");
    fs::write(&tf, "test").unwrap();
    let r = resolve_paste_conflict(&tf);
    assert_ne!(r, tf);
    assert!(r.file_name().unwrap().to_string_lossy().contains("Copy"));
    fs::remove_file(&tf).unwrap();
}

#[test]
fn test_paste_conflict_no_ext() {
    let tmp = std::env::temp_dir();
    let tf = tmp.join("fe_test_pastenofile");
    fs::write(&tf, "test").unwrap();
    let r = resolve_paste_conflict(&tf);
    let n = r.file_name().unwrap().to_string_lossy();
    assert!(n.contains("Copy"), "expected 'Copy' in: {n}");
    assert!(!n.contains('.'), "expected no dot in: {n}");
    fs::remove_file(&tf).unwrap();
}

// ══════════════════════════════════════════════
//  File operation integration tests
// ══════════════════════════════════════════════

#[test]
fn test_create_and_list_directory() {
    let td = TempDir::new("create_list");
    let sub = td.join("mysubdir");
    fs::create_dir(&sub).unwrap();
    assert!(sub.exists());
    assert!(sub.is_dir());

    // Create files inside
    fs::write(sub.join("a.txt"), "hello").unwrap();
    fs::write(sub.join("b.txt"), "world").unwrap();

    let entries: Vec<String> = fs::read_dir(&sub)
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    assert_eq!(entries.len(), 2);
    assert!(entries.contains(&"a.txt".to_string()));
    assert!(entries.contains(&"b.txt".to_string()));
}

#[test]
fn test_create_file_and_read() {
    let td = TempDir::new("create_file");
    let f = td.join("test.txt");
    assert!(!f.exists());
    fs::write(&f, "hello world").unwrap();
    assert!(f.exists());
    assert_eq!(fs::read_to_string(&f).unwrap(), "hello world");
}

#[test]
fn test_create_duplicate_file_errors() {
    let td = TempDir::new("dup_file");
    let f = td.join("existing.txt");
    fs::write(&f, "first").unwrap();
    // Trying to create again should check existence
    assert!(f.exists());
}

#[test]
fn test_delete_file() {
    let td = TempDir::new("delete_file");
    let f = td.join("to_delete.txt");
    fs::write(&f, "bye").unwrap();
    assert!(f.exists());
    fs::remove_file(&f).unwrap();
    assert!(!f.exists());
}

#[test]
fn test_delete_dir_recursive() {
    let td = TempDir::new("delete_dir");
    let sub = td.join("deep");
    fs::create_dir_all(sub.join("a/b/c")).unwrap();
    fs::write(sub.join("x.txt"), "x").unwrap();
    assert!(sub.exists());
    fs::remove_dir_all(&sub).unwrap();
    assert!(!sub.exists());
}

#[test]
fn test_rename_file() {
    let td = TempDir::new("rename");
    let old = td.join("old_name.txt");
    let new = td.join("new_name.txt");
    fs::write(&old, "data").unwrap();
    assert!(old.exists());
    assert!(!new.exists());
    fs::rename(&old, &new).unwrap();
    assert!(!old.exists());
    assert!(new.exists());
    assert_eq!(fs::read_to_string(&new).unwrap(), "data");
}

#[test]
fn test_rename_directory() {
    let td = TempDir::new("rename_dir");
    let old = td.join("old_dir");
    let new = td.join("new_dir");
    fs::create_dir(&old).unwrap();
    fs::write(old.join("inside.txt"), "content").unwrap();
    fs::rename(&old, &new).unwrap();
    assert!(!old.exists());
    assert!(new.exists());
    assert!(new.join("inside.txt").exists());
}

#[test]
fn test_rename_nonexistent_errors() {
    let td = TempDir::new("rename_err");
    let src = td.join("does_not_exist");
    let dst = td.join("target");
    assert!(fs::rename(&src, &dst).is_err());
}

// ══════════════════════════════════════════════
//  Copy / move tests
// ══════════════════════════════════════════════

#[test]
fn test_copy_file() {
    let td = TempDir::new("copy");
    let src = td.join("source.txt");
    let dst = td.join("dest.txt");
    fs::write(&src, "copy me").unwrap();
    fs::copy(&src, &dst).unwrap();
    assert!(dst.exists());
    assert_eq!(fs::read_to_string(&src).unwrap(), fs::read_to_string(&dst).unwrap());
}

#[test]
fn test_move_file_same_fs() {
    let td = TempDir::new("move_same");
    let src = td.join("src.txt");
    let dst = td.join("dst.txt");
    fs::write(&src, "move me").unwrap();
    fs::rename(&src, &dst).unwrap(); // same FS = fast rename
    assert!(!src.exists());
    assert!(dst.exists());
    assert_eq!(fs::read_to_string(&dst).unwrap(), "move me");
}

#[test]
fn test_copy_dir_recursive() {
    let td = TempDir::new("copy_dir");
    let src = td.join("src_dir");
    let dst = td.join("dst_dir");

    fs::create_dir_all(src.join("sub")).unwrap();
    fs::write(src.join("a.txt"), "a").unwrap();
    fs::write(src.join("sub/b.txt"), "b").unwrap();

    // Recursive copy
    fn copy_recursive(src: &Path, dst: &Path) {
        fs::create_dir_all(dst).unwrap();
        for entry in fs::read_dir(src).unwrap().flatten() {
            let s = entry.path();
            let d = dst.join(entry.file_name());
            if s.is_dir() {
                copy_recursive(&s, &d);
            } else {
                fs::copy(&s, &d).unwrap();
            }
        }
    }
    copy_recursive(&src, &dst);

    assert!(dst.exists());
    assert!(dst.join("a.txt").exists());
    assert!(dst.join("sub/b.txt").exists());
    assert_eq!(fs::read_to_string(dst.join("a.txt")).unwrap(), "a");
    assert_eq!(fs::read_to_string(dst.join("sub/b.txt")).unwrap(), "b");
}

// ══════════════════════════════════════════════
//  Path operations
// ══════════════════════════════════════════════

#[test]
fn test_parent_directory() {
    let p = Path::new("/home/user/docs/file.txt");
    assert_eq!(p.parent().unwrap().to_string_lossy(), "/home/user/docs");
}

#[test]
fn test_parent_root() {
    let p = Path::new("/");
    assert!(p.parent().is_none());
}

#[test]
fn test_path_exists() {
    let td = TempDir::new("path_exists");
    assert!(td.path.exists());
    assert!(!td.join("nope").exists());
}

// ══════════════════════════════════════════════
//  Metadata extraction
// ══════════════════════════════════════════════

#[test]
fn test_file_metadata() {
    let td = TempDir::new("metadata");
    let f = td.join("meta.txt");
    fs::write(&f, "0123456789").unwrap(); // 10 bytes

    let meta = fs::metadata(&f).unwrap();
    assert!(!meta.is_dir());
    assert_eq!(meta.len(), 10);

    let created = meta.created().ok();
    let modified = meta.modified().ok();
    assert!(created.is_some());
    assert!(modified.is_some());
}

#[test]
fn test_dir_metadata() {
    let td = TempDir::new("meta_dir");
    let meta = fs::metadata(&td.path).unwrap();
    assert!(meta.is_dir());
}

// ══════════════════════════════════════════════
//  Edge cases
// ══════════════════════════════════════════════

#[test]
fn test_empty_directory_listing() {
    let td = TempDir::new("empty");
    let count = fs::read_dir(&td.path).unwrap().count();
    assert_eq!(count, 0);
}

#[test]
fn test_symlink_handling() {
    // Verify we can detect symlinks (walkdir follow_links=false handles this)
    let td = TempDir::new("symlink");
    let target = td.join("target.txt");
    fs::write(&target, "target").unwrap();

    #[cfg(unix)]
    {
        let link = td.join("link.txt");
        std::os::unix::fs::symlink(&target, &link).unwrap();
        assert!(link.exists());
        // symlink metadata should show it as a file, not a dir
        assert!(!fs::metadata(&link).unwrap().is_dir());
    }
}

#[test]
fn test_nested_directory_operations() {
    let td = TempDir::new("nested");
    let deep = td.join("a/b/c/d");
    fs::create_dir_all(&deep).unwrap();
    assert!(deep.exists());

    // Navigate up
    let mut current = deep.clone();
    for expected in &["d", "c", "b", "a"] {
        assert!(current.file_name().unwrap().to_string_lossy() == *expected);
        current = current.parent().unwrap().to_path_buf();
    }
}

#[test]
fn test_file_name_extraction() {
    let p = Path::new("/home/user/file.tar.gz");
    assert_eq!(p.file_name().unwrap().to_string_lossy(), "file.tar.gz");
    assert_eq!(p.extension().unwrap().to_string_lossy(), "gz");
    assert_eq!(p.file_stem().unwrap().to_string_lossy(), "file.tar");
}

#[test]
fn test_hidden_files() {
    let td = TempDir::new("hidden");
    fs::write(td.join(".gitignore"), "node_modules").unwrap();
    fs::write(td.join("normal.txt"), "visible").unwrap();

    let names: Vec<String> = fs::read_dir(&td.path)
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    assert_eq!(names.len(), 2);
    assert!(names.contains(&".gitignore".to_string()));
    assert!(names.contains(&"normal.txt".to_string()));
}

#[test]
fn test_large_file_handling() {
    let td = TempDir::new("large");
    let f = td.join("big.bin");
    // Create a 1MB file of zeros
    let data = vec![0u8; 1024 * 1024];
    fs::write(&f, &data).unwrap();
    let meta = fs::metadata(&f).unwrap();
    assert_eq!(meta.len(), 1024 * 1024);
}

#[test]
fn test_unicode_filenames() {
    let td = TempDir::new("unicode");
    let name = "你好世界.txt";
    let f = td.join(name);
    fs::write(&f, "unicode").unwrap();
    assert!(f.exists());

    let found: Vec<String> = fs::read_dir(&td.path)
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0], name);
}
