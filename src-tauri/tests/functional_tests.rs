use std::path::PathBuf;

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

fn resolve_paste_conflict(path: &std::path::Path) -> std::path::PathBuf {
    if !path.exists() {
        return path.to_path_buf();
    }
    let parent = path.parent().unwrap_or(std::path::Path::new("."));
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
                format!("{} ({}).{}", stem, counter, ext)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wildcard_exact() {
        assert!(wildcard_match("hello.txt", "hello.txt"));
        assert!(!wildcard_match("hello.txt", "world.txt"));
    }
    #[test]
    fn wildcard_star() {
        assert!(wildcard_match("*.txt", "hello.txt"));
        assert!(wildcard_match("*.txt", "HELLO.TXT"));
        assert!(!wildcard_match("*.txt", "hello.md"));
        assert!(wildcard_match("*test*", "mytestfile"));
        assert!(wildcard_match("*", "anything"));
    }
    #[test]
    fn wildcard_question() {
        assert!(wildcard_match("file?.txt", "file1.txt"));
        assert!(!wildcard_match("file?.txt", "file12.txt"));
        assert!(wildcard_match("???.txt", "abc.txt"));
    }
    #[test]
    fn wildcard_empty() {
        assert!(wildcard_match("", ""));
        assert!(!wildcard_match("", "x"));
    }

    #[test]
    fn parse_size_greater() {
        assert_eq!(parse_size_filter(">1MB"), Some(('>', 1048576)));
        assert_eq!(parse_size_filter(">100KB"), Some(('>', 102400)));
    }
    #[test]
    fn parse_size_less() {
        assert_eq!(parse_size_filter("<10KB"), Some(('<', 10240)));
    }
    #[test]
    fn parse_size_invalid() {
        assert_eq!(parse_size_filter("1MB"), None);
        assert_eq!(parse_size_filter("abc"), None);
    }

    #[test]
    fn condition_substring() {
        assert!(condition_matches_file("hello", "hello.txt", 100));
        assert!(!condition_matches_file("xyz", "hello.txt", 100));
    }
    #[test]
    fn condition_wildcard() {
        assert!(condition_matches_file("*.rs", "main.rs", 100));
    }
    #[test]
    fn condition_size() {
        assert!(condition_matches_file(">1MB", "big.txt", 2000000));
        assert!(!condition_matches_file(">1MB", "s.txt", 500000));
    }
    #[test]
    fn condition_empty() {
        assert!(!condition_matches_file("", "t.txt", 100));
    }

    #[test]
    fn paste_no_conflict() {
        let tmp = std::env::temp_dir().join("tauri_test_noexist_xyz_12345");
        assert_eq!(resolve_paste_conflict(&tmp), tmp);
    }
    #[test]
    fn paste_conflict_ext() {
        let tmp = std::env::temp_dir();
        let tf = tmp.join("tauri_test_pasteconf.txt");
        std::fs::write(&tf, "test").unwrap();
        let r = resolve_paste_conflict(&tf);
        assert_ne!(r, tf);
        assert!(r.file_name().unwrap().to_string_lossy().contains("Copy"));
        std::fs::remove_file(&tf).unwrap();
    }
    #[test]
    fn paste_conflict_no_ext() {
        let tmp = std::env::temp_dir();
        let tf = tmp.join("tauri_test_pastenofile");
        std::fs::write(&tf, "test").unwrap();
        let r = resolve_paste_conflict(&tf);
        let n = r.file_name().unwrap().to_string_lossy();
        assert!(n.contains("Copy"), "got: {n}");
        assert!(!n.contains('.'), "got: {n}");
        std::fs::remove_file(&tf).unwrap();
    }
}
