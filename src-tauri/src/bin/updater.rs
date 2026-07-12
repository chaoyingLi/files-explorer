// updater.rs — 独立更新器
// 双路下载(GitHub+Gitee竞速) → 安装 → 启动新版本
// 用法: updater <版本号>

use std::env;
use std::path::{Path, PathBuf};
use std::process::{exit, Command};
use std::thread;
use std::time::Duration;

const GITHUB_REPO: &str = "chaoyingLi/files-explorer";
const GITEE_REPO: &str = "hhyd/files-explorer";

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("用法: updater <版本号> [主进程PID]");
        exit(1);
    }

    let version = args[1].clone();
    let app_path = app_bundle_path();

    println!("╔════════════════════════════════╗");
    println!("║  Files Explorer 更新器        ║");
    println!("╚════════════════════════════════╝");
    println!("版本: {version}");
    println!("目标: {}", app_path.display());

    // 1. 双路下载
    // 1. 下载
    let filename = download_filename(&version);
    println!("\n📥 下载: {filename}");

    let github_url =
        format!("https://github.com/{GITHUB_REPO}/releases/download/v{version}/{filename}");
    let gitee_url =
        format!("https://gitee.com/{GITEE_REPO}/releases/download/v{version}/{filename}");

    let tmp_dir = std::env::temp_dir().join("files-explorer-update");
    let _ = std::fs::create_dir_all(&tmp_dir);
    let local_file = tmp_dir.join(&filename);

    println!("  源1: {github_url}");
    println!("  源2: {gitee_url}");
    println!("  目标: {}", local_file.display());

    if !download_race(&github_url, &gitee_url, &local_file) {
        eprintln!("\n❌ 下载失败");
        exit(1);
    }

    // 2. 等待主进程退出
    println!("\n⏳ 等待主进程退出 (3秒)...");
    thread::sleep(Duration::from_secs(3));

    // 3. 安装
    println!("\n📦 安装...");
    match platform_install(&local_file, &app_path) {
        Ok(()) => println!("   ✅ 完成"),
        Err(e) => {
            eprintln!("   ❌ 安装失败: {e}");
            exit(1);
        }
    }

    // 4. 清理 + 启动
    let _ = std::fs::remove_dir_all(&tmp_dir);
    println!("\n🚀 启动新版本...");
    launch_app(&app_path);
    println!("✅ 更新完成");
}

// ── 文件名 ──
fn download_filename(version: &str) -> String {
    let platform = if cfg!(target_os = "macos") {
        "darwin"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else {
        "linux"
    };
    let arch = if cfg!(target_arch = "x86_64") {
        "x64"
    } else {
        "aarch64"
    };

    match (platform, arch) {
        ("darwin", "x64") => format!("Files.Explorer_{version}_x64.dmg"),
        ("darwin", "aarch64") => format!("Files.Explorer_{version}_aarch64.dmg"),
        ("windows", "x64") => format!("Files.Explorer_{version}_x64_en-US.msi"),
        ("linux", "x64") => format!("Files.Explorer_{version}_amd64.AppImage"),
        _ => format!("Files.Explorer_{version}_{}.dmg", std::env::consts::OS),
    }
}

// ── 双路竞速下载（两个线程同时下，谁先成用谁） ──
fn download_race(url1: &str, url2: &str, dest: &Path) -> bool {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    let done = Arc::new(AtomicBool::new(false));
    let d1 = Arc::clone(&done);
    let d2 = Arc::clone(&done);
    let (u1, u2) = (url1.to_string(), url2.to_string());
    let (dest1, dest2) = (dest.to_path_buf(), dest.to_path_buf());

    let h1 = thread::spawn(move || {
        if download_file(&u1, &dest1) {
            d1.store(true, Ordering::SeqCst);
        }
    });
    let h2 = thread::spawn(move || {
        if download_file(&u2, &dest2) {
            d2.store(true, Ordering::SeqCst);
        }
    });

    let _ = h1.join();
    let _ = h2.join();

    done.load(Ordering::SeqCst)
}

fn download_file(url: &str, dest: &Path) -> bool {
    let status = if cfg!(target_os = "windows") {
        Command::new("powershell")
            .args([
                "-Command",
                &format!(
                    "Invoke-WebRequest -Uri '{}' -OutFile '{}' -TimeoutSec 600",
                    url,
                    dest.display()
                ),
            ])
            .status()
    } else {
        Command::new("curl")
            .args([
                "-fSL",
                "--connect-timeout",
                "10",
                "--max-time",
                "600",
                "-o",
                &dest.to_string_lossy(),
                url,
            ])
            .status()
    };

    match status {
        Ok(s) if s.success() => {
            println!("   ✅ {}", url);
            true
        }
        _ => {
            println!("   ❌ {}", url);
            false
        }
    }
}

// ── 进程检测 ──
// ── 应用路径 ──
#[cfg(target_os = "macos")]
fn app_bundle_path() -> PathBuf {
    PathBuf::from("/Applications/Files Explorer.app")
}
#[cfg(target_os = "windows")]
fn app_bundle_path() -> PathBuf {
    std::env::current_exe().unwrap_or_else(|_| PathBuf::from("Files Explorer.exe"))
}
#[cfg(target_os = "linux")]
fn app_bundle_path() -> PathBuf {
    PathBuf::from("/usr/local/bin/files-explorer")
}

// ── 平台安装 ──
#[cfg(target_os = "macos")]
fn platform_install(dmg: &Path, app_path: &Path) -> Result<(), String> {
    // 挂载 DMG
    let output = Command::new("hdiutil")
        .args(["attach", "-nobrowse", "-readonly"])
        .arg(dmg)
        .output()
        .map_err(|e| format!("挂载失败: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mount = stdout
        .lines()
        .last()
        .and_then(|l| l.split('\t').last())
        .map(|s| s.trim().to_string())
        .ok_or("无法获取挂载点")?;

    let src = Path::new(&mount).join("Files Explorer.app");
    if !src.exists() {
        let _ = Command::new("hdiutil").args(["detach", &mount]).status();
        return Err("DMG 中未找到 .app".into());
    }

    // 删旧换新
    if app_path.exists() {
        let _ = std::fs::remove_dir_all(app_path);
    }
    Command::new("cp")
        .args(["-R", &src.to_string_lossy(), &app_path.to_string_lossy()])
        .status()
        .map_err(|e| format!("复制失败: {e}"))?;

    // 清除隔离
    let _ = Command::new("xattr")
        .args(["-cr", &app_path.to_string_lossy()])
        .status();
    let _ = Command::new("hdiutil").args(["detach", &mount]).status();
    Ok(())
}

#[cfg(target_os = "windows")]
fn platform_install(msi: &Path, _app_path: &Path) -> Result<(), String> {
    Command::new("msiexec")
        .args(["/i", &msi.to_string_lossy(), "/quiet", "/norestart"])
        .status()
        .map_err(|e| format!("安装失败: {e}"))?;
    Ok(())
}

#[cfg(target_os = "linux")]
fn platform_install(pkg: &Path, _app_path: &Path) -> Result<(), String> {
    let dest = Path::new("/usr/local/bin/files-explorer");
    Command::new("cp")
        .args(["-f", &pkg.to_string_lossy(), &dest.to_string_lossy()])
        .status()
        .map_err(|e| format!("复制失败: {e}"))?;
    Command::new("chmod")
        .args(["+x", &dest.to_string_lossy()])
        .status()
        .map_err(|e| format!("chmod 失败: {e}"))?;
    Ok(())
}

// ── 启动 ──
fn launch_app(app_path: &Path) {
    #[cfg(target_os = "macos")]
    let _ = Command::new("open").arg(app_path).spawn();
    #[cfg(target_os = "windows")]
    let _ = Command::new(app_path).spawn();
    #[cfg(target_os = "linux")]
    let _ = Command::new(app_path).spawn();
    thread::sleep(Duration::from_secs(1));
}
