use crate::types::{DiskInfo, SpecialDirs};
#[cfg(target_os = "windows")]
use std::path::Path;

pub fn get_drives() -> Result<Vec<DiskInfo>, String> {
    let mut drives = Vec::new();

    #[cfg(target_os = "windows")]
    {
        for letter in b'A'..=b'Z' {
            let drive = format!("{}:\\", letter as char);
            let path = Path::new(&drive);
            if !path.exists() {
                continue;
            }

            let mut total: u64 = 0;
            let mut free: u64 = 0;
            let label = get_windows_volume_label(&drive);

            unsafe {
                let path_wide: Vec<u16> = drive.encode_utf16().chain(std::iter::once(0)).collect();
                if GetDiskFreeSpaceExW(
                    path_wide.as_ptr(),
                    std::ptr::null_mut(),
                    &mut total,
                    &mut free,
                ) == 0
                {
                    total = 0;
                    free = 0;
                }
            }

            let fs = get_windows_filesystem(&drive);
            drives.push(DiskInfo {
                name: drive.clone(),
                mount_point: drive.clone(),
                total_space: total,
                available_space: free,
                used_space: total.saturating_sub(free),
                file_system: fs,
                label,
            });
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        // Root volume always present
        if let Ok(info) = get_unix_disk_info("/") {
            drives.push(info);
        }

        #[cfg(target_os = "macos")]
        {
            // Enumerate external volumes mounted under /Volumes
            if let Ok(entries) = std::fs::read_dir("/Volumes") {
                for entry in entries.flatten() {
                    let vol_path = entry.path();
                    if !vol_path.is_dir() {
                        continue;
                    }
                    let vol_str = vol_path.to_string_lossy().to_string();
                    // Skip the root volume itself (already added above)
                    if vol_str == "/" {
                        continue;
                    }
                    if let Ok(mut info) = get_unix_disk_info(&vol_str) {
                        // Use the volume name from the mount point
                        let vol_name = vol_path
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_else(|| vol_str.clone());
                        info.label = vol_name.clone();
                        if info.name == vol_str && vol_name != "/" {
                            info.name = vol_name;
                        }
                        drives.push(info);
                    }
                }
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            // Linux: add home volume if different from root
            let home = std::env::var("HOME").unwrap_or_else(|_| "/".to_string());
            if home != "/" {
                if let Ok(info) = get_unix_disk_info(&home) {
                    drives.push(info);
                }
            }
        }

        if drives.is_empty() {
            drives.push(DiskInfo {
                name: "Root".to_string(),
                mount_point: "/".to_string(),
                total_space: 0,
                available_space: 0,
                used_space: 0,
                file_system: "unknown".to_string(),
                label: "System".to_string(),
            });
        }
    }

    Ok(drives)
}

#[cfg(target_os = "windows")]
extern "system" {
    fn GetDiskFreeSpaceExW(
        lpDirectoryName: *const u16,
        lpFreeBytesAvailableToCaller: *mut u64,
        lpTotalNumberOfBytes: *mut u64,
        lpTotalNumberOfFreeBytes: *mut u64,
    ) -> i32;

    fn GetVolumeInformationW(
        lpRootPathName: *const u16,
        lpVolumeNameBuffer: *mut u16,
        nVolumeNameSize: u32,
        lpVolumeSerialNumber: *mut u32,
        lpMaximumComponentLength: *mut u32,
        lpFileSystemFlags: *mut u32,
        lpFileSystemNameBuffer: *mut u16,
        nFileSystemNameSize: u32,
    ) -> i32;
}

#[cfg(target_os = "windows")]
pub fn get_windows_volume_label(drive: &str) -> String {
    let drive_wide: Vec<u16> = drive.encode_utf16().chain(std::iter::once(0)).collect();
    let mut buf = vec![0u16; 128];
    unsafe {
        if GetVolumeInformationW(
            drive_wide.as_ptr(),
            buf.as_mut_ptr(),
            buf.len() as u32,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            0,
        ) != 0
        {
            let end = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
            return String::from_utf16_lossy(&buf[..end]);
        }
    }
    String::new()
}

#[cfg(target_os = "windows")]
pub fn get_windows_filesystem(drive: &str) -> String {
    let drive_wide: Vec<u16> = drive.encode_utf16().chain(std::iter::once(0)).collect();
    let mut buf = vec![0u16; 32];
    unsafe {
        if GetVolumeInformationW(
            drive_wide.as_ptr(),
            std::ptr::null_mut(),
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            buf.as_mut_ptr(),
            buf.len() as u32,
        ) != 0
        {
            let end = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
            return String::from_utf16_lossy(&buf[..end]);
        }
    }
    String::new()
}

#[cfg(not(target_os = "windows"))]
pub fn get_unix_disk_info(path: &str) -> Result<DiskInfo, String> {
    use std::ffi::{CStr, CString};
    let cpath = CString::new(path).map_err(|e| e.to_string())?;
    unsafe {
        let mut stat: libc::statvfs = std::mem::zeroed();
        if libc::statvfs(cpath.as_ptr(), &mut stat) != 0 {
            return Err("statvfs failed".to_string());
        }
        let total = stat.f_frsize as u64 * stat.f_blocks as u64;
        let free = stat.f_frsize as u64 * stat.f_bavail as u64;
        // Get real filesystem type via statfs
        let mut sfs: libc::statfs = std::mem::zeroed();
        let file_system = if libc::statfs(cpath.as_ptr(), &mut sfs) == 0 {
            CStr::from_ptr(sfs.f_fstypename.as_ptr())
                .to_string_lossy()
                .to_string()
        } else {
            String::from("unknown")
        };
        Ok(DiskInfo {
            name: path.to_string(),
            mount_point: path.to_string(),
            total_space: total,
            available_space: free,
            used_space: total.saturating_sub(free),
            file_system,
            label: String::new(),
        })
    }
}

pub fn get_special_dirs() -> Result<SpecialDirs, String> {
    let home = dirs::home_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| {
            if cfg!(target_os = "windows") {
                std::env::var("USERPROFILE").unwrap_or_else(|_| String::from("C:\\"))
            } else {
                std::env::var("HOME").unwrap_or_else(|_| String::from("/"))
            }
        });

    let path_to_str = |p: Option<std::path::PathBuf>| -> String {
        p.map(|x| x.to_string_lossy().to_string())
            .unwrap_or_default()
    };

    Ok(SpecialDirs {
        home: home.clone(),
        desktop: path_to_str(dirs::desktop_dir()),
        documents: path_to_str(dirs::document_dir()),
        downloads: path_to_str(dirs::download_dir()),
        pictures: path_to_str(dirs::picture_dir()),
        music: path_to_str(dirs::audio_dir()),
        videos: path_to_str(dirs::video_dir()),
    })
}
