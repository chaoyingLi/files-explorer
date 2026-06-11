// Standalone test runner - no Tauri dependencies
// Run with: rustc --test functional_tests_standalone.rs -o test_runner.exe && ./test_runner

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
    let rest = &s[1..].trim();
    let num_end = rest
        .find(|c: char| !c.is_ascii_digit() && c != '.')
        .unwrap_or(rest.len());
    if num_end == 0 {
        return None;
    }
    let num: f64 = rest[..num_end].parse().ok()?;
    let unit = rest[num_end..].trim().to_lowercase();
    let m = match unit.as_str() {
        "b" | "" => 1.0,
        "k" | "kb" => 1024.0,
        "m" | "mb" => 1024.0_f64.powi(2),
        "g" | "gb" => 1024.0_f64.powi(3),
        "t" | "tb" => 1024.0_f64.powi(4),
        _ => return None,
    };
    Some((op, (num * m) as u64))
}

fn condition_matches_file(cond: &str, name: &str, size: u64) -> bool {
    let cond = cond.trim();
    if cond.is_empty() {
        return false;
    }
    if let Some((op, th)) = parse_size_filter(cond) {
        return match op {
            '>' => size > th,
            '<' => size < th,
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
    let mut c: u32 = 1;
    loop {
        let name = if c == 1 {
            if ext.is_empty() {
                format!("{stem} - Copy")
            } else {
                format!("{stem} - Copy.{ext}")
            }
        } else {
            if ext.is_empty() {
                format!("{stem} ({c})")
            } else {
                format!("{stem} ({c}).{ext}")
            }
        };
        let candidate = parent.join(&name);
        if !candidate.exists() {
            return candidate;
        }
        c += 1;
        if c > 999 {
            break;
        }
    }
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    parent.join(format!("{stem}_{ts}.{ext}"))
}

fn main() {
    let mut passed = 0;
    let mut failed = 0;
    macro_rules! check {
        ($cond:expr, $name:expr) => {
            if $cond {
                passed += 1;
                println!("  PASS: {}", $name);
            } else {
                failed += 1;
                println!("  FAIL: {}", $name);
            }
        };
    }

    println!("\n=== wildcard_match ===");
    check!(wildcard_match("hello.txt", "hello.txt"), "exact match");
    check!(!wildcard_match("hello.txt", "world.txt"), "mismatch");
    check!(
        wildcard_match("*.txt", "hello.txt"),
        "*.txt matches hello.txt"
    );
    check!(
        wildcard_match("*.txt", "HELLO.TXT"),
        "case-insensitive *.txt"
    );
    check!(!wildcard_match("*.txt", "hello.md"), "*.txt rejects .md");
    check!(wildcard_match("*test*", "mytestfile"), "*test* contains");
    check!(wildcard_match("*", "anything"), "star matches all");
    check!(wildcard_match("*", ""), "star matches empty");
    check!(wildcard_match("file?.txt", "file1.txt"), "? single char");
    check!(
        !wildcard_match("file?.txt", "file12.txt"),
        "? rejects double"
    );
    check!(wildcard_match("", ""), "empty matches empty");
    check!(!wildcard_match("", "x"), "empty rejects non-empty");

    println!("\n=== parse_size_filter ===");
    check!(parse_size_filter(">1MB") == Some(('>', 1048576)), ">1MB");
    check!(parse_size_filter(">100KB") == Some(('>', 102400)), ">100KB");
    check!(parse_size_filter(">2GB") == Some(('>', 2147483648)), ">2GB");
    check!(parse_size_filter("<10KB") == Some(('<', 10240)), "<10KB");
    check!(parse_size_filter(">500B") == Some(('>', 500)), ">500B");
    check!(parse_size_filter("1MB") == None, "no op prefix");
    check!(parse_size_filter("abc") == None, "garbage");

    println!("\n=== condition_matches_file ===");
    check!(
        condition_matches_file("hello", "hello.txt", 100),
        "substring"
    );
    check!(!condition_matches_file("xyz", "hello.txt", 100), "no match");
    check!(
        condition_matches_file("*.rs", "main.rs", 100),
        "wildcard match"
    );
    check!(
        !condition_matches_file("*.rs", "main.py", 100),
        "wildcard reject"
    );
    check!(
        condition_matches_file(">1MB", "big.txt", 2_000_000),
        "size >1MB true"
    );
    check!(
        !condition_matches_file(">1MB", "small.txt", 500_000),
        "size >1MB false"
    );
    check!(
        condition_matches_file("<1MB", "small.txt", 500_000),
        "size <1MB true"
    );
    check!(!condition_matches_file("", "t.txt", 100), "empty condition");

    println!("\n=== resolve_paste_conflict ===");
    let tmp = std::env::temp_dir();
    let noexist = tmp.join("tauri_test_noexist_xyz_12345");
    check!(
        resolve_paste_conflict(&noexist) == noexist,
        "no conflict same path"
    );

    let tf = tmp.join("tauri_test_conflict.txt");
    std::fs::write(&tf, "test").unwrap();
    let r = resolve_paste_conflict(&tf);
    check!(r != tf, "conflict gives different path");
    check!(r.to_string_lossy().contains("Copy"), "contains 'Copy'");
    check!(
        r.extension().map(|e| e == "txt").unwrap_or(false),
        "keeps extension"
    );
    std::fs::remove_file(&tf).unwrap();

    let tf2 = tmp.join("tauri_test_nofile");
    std::fs::write(&tf2, "test").unwrap();
    let r2 = resolve_paste_conflict(&tf2);
    check!(
        r2.file_name().unwrap().to_string_lossy().contains("Copy"),
        "no-ext has Copy"
    );
    check!(
        !r2.file_name().unwrap().to_string_lossy().contains('.'),
        "no-ext no dot"
    );
    std::fs::remove_file(&tf2).unwrap();

    println!("\n=== Results: {passed} passed, {failed} failed ===");
    if failed > 0 {
        std::process::exit(1);
    }
}
