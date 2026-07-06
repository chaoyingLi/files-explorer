fn main() {
    #[cfg(target_os = "linux")]
    check_linux_deps();
    tauri_build::build()
}

#[cfg(target_os = "linux")]
fn check_linux_deps() {
    let required: &[(&str, &str)] = &[
        ("webkit2gtk-4.1", "libwebkit2gtk-4.1-dev"),
        ("gtk+-3.0", "libgtk-3-dev"),
        ("libappindicator3", "libappindicator3-dev"),
        ("librsvg-2.0", "librsvg2-dev"),
    ];

    let mut missing = false;
    for (pkg, deb_pkg) in required {
        let status = std::process::Command::new("pkg-config")
            .args(["--exists", pkg])
            .status();
        match status {
            Ok(s) if s.success() => {}
            _ => {
                if !missing {
                    eprintln!();
                    eprintln!("  ═══════════════════════════════════════════════");
                    eprintln!("  ⚠  Missing Linux system dependencies detected");
                    eprintln!("  ═══════════════════════════════════════════════");
                    missing = true;
                }
                eprintln!("  ✗ {:<30}  sudo apt install {}", pkg, deb_pkg);
            }
        }
    }
    if missing {
        eprintln!("  ═══════════════════════════════════════════════");
        eprintln!();
    }
}
