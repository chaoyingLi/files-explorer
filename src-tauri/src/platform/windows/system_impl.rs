// platform/windows/system_impl.rs
// Windows system capability implementation.

use crate::core::types::DiskInfo;
use crate::platform::PlatformSystem;
use std::path::{Path, PathBuf};

pub struct SystemImpl;

impl PlatformSystem for SystemImpl {
    fn open_file(&self, path: &Path) -> Result<(), String> {
        opener::open(path).map_err(|e| format!("Failed to open: {}", e))
    }

    fn open_terminal(&self, dir: &Path) -> Result<(), String> {
        // Prefer Windows Terminal; fall back to cmd.exe
        let native = dir.to_string_lossy().replace("/", "\\");
        let wt = std::process::Command::new("wt")
            .args(["-d"])
            .arg(&native)
            .spawn();
        if wt.is_err() {
            std::process::Command::new("cmd")
                .args(["/K", "pushd"])
                .arg(&native)
                .spawn()
                .map_err(|e| format!("Failed to open terminal: {}", e))?;
        }
        Ok(())
    }

    fn show_in_file_manager(&self, path: &Path) -> Result<(), String> {
        let native = path.to_string_lossy().replace("/", "\\");
        std::process::Command::new("explorer")
            .arg(format!("/select,{}", native))
            .spawn()
            .map_err(|e| format!("Failed: {}", e))?;
        Ok(())
    }

    fn show_properties(&self, path: &Path) -> Result<(), String> {
        if !path.exists() {
            return Err(format!("Path does not exist: {}", path.display()));
        }
        let fp: Vec<u16> = path
            .to_string_lossy()
            .replace("/", "\\")
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        unsafe {
            let ret = SHObjectProperties(
                std::ptr::null_mut(),
                0x2, // SHOP_FILEPATH
                fp.as_ptr(),
                std::ptr::null(),
            );
            if ret == 0 {
                return Err("SHObjectProperties failed".to_string());
            }
        }
        Ok(())
    }

    fn print_file(&self, path: &Path) -> Result<(), String> {
        let native = path.to_string_lossy().replace("/", "\\");
        std::process::Command::new("print")
            .arg(&native)
            .spawn()
            .map_err(|e| format!("Print: {}", e))?;
        Ok(())
    }

    fn get_file_icon(&self, path: &Path) -> Result<Vec<u8>, String> {
        get_windows_icon_png(path)
    }

    fn set_auto_start(&self, enabled: bool) -> Result<(), String> {
        let exe = std::env::current_exe()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        if enabled {
            let _ = std::process::Command::new("reg")
                .args([
                    "add",
                    r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
                    "/v",
                    "FilesExplorer",
                    "/d",
                    &exe,
                    "/f",
                ])
                .output();
        } else {
            let _ = std::process::Command::new("reg")
                .args([
                    "delete",
                    r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
                    "/v",
                    "FilesExplorer",
                    "/f",
                ])
                .output();
        }
        Ok(())
    }

    fn is_auto_start_enabled(&self) -> bool {
        std::process::Command::new("reg")
            .args([
                "query",
                r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
                "/v",
                "FilesExplorer",
            ])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    fn write_clipboard(&self, paths: &[PathBuf]) {
        write_windows_clipboard(paths);
    }

    fn read_clipboard(&self) -> Option<Vec<PathBuf>> {
        read_windows_clipboard()
    }

    fn start_native_drag(&self, paths: &[String]) -> Result<String, String> {
        native_drag_impl(paths)
    }

    fn get_drives(&self) -> Vec<DiskInfo> {
        get_windows_drives()
    }

    fn is_tray_supported(&self) -> bool {
        true // Windows system tray always works
    }

    fn send_notification(&self, _title: &str, _body: &str) -> Result<(), String> {
        // Windows toast notifications: needs winrt-notification or similar crate.
        // For now: no-op (the frontend already handles in-app notifications).
        Ok(())
    }
}

impl SystemImpl {
    pub fn instance() -> &'static SystemImpl {
        &SystemImpl
    }
}

extern "system" {
    fn SHObjectProperties(
        hwnd: *mut std::ffi::c_void,
        dwType: u32,
        pszName: *const u16,
        pszParameters: *const u16,
    ) -> i32;
}

// ── Windows icon extraction ──
fn get_windows_icon_png(path: &Path) -> Result<Vec<u8>, String> {
    use image::codecs::png::PngEncoder;
    use image::{ColorType, ImageEncoder};
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    const SHGFI_ICON: u32 = 0x000000100;
    const SHGFI_LARGEICON: u32 = 0x000000000;
    const SHGFI_USEFILEATTRIBUTES: u32 = 0x000000010;
    const ICON_SIZE: i32 = 32;

    #[allow(non_snake_case)]
    #[repr(C)]
    struct SHFILEINFOW {
        hIcon: *mut std::ffi::c_void,
        iIcon: i32,
        dwAttributes: u32,
        szDisplayName: [u16; 260],
        szTypeName: [u16; 80],
    }

    #[allow(non_snake_case)]
    #[repr(C)]
    struct BITMAPINFOHEADER {
        biSize: u32,
        biWidth: i32,
        biHeight: i32,
        biPlanes: u16,
        biBitCount: u16,
        biCompression: u32,
        biSizeImage: u32,
        biXPelsPerMeter: i32,
        biYPelsPerMeter: i32,
        biClrUsed: u32,
        biClrImportant: u32,
    }

    extern "system" {
        fn SHGetFileInfoW(
            pszPath: *const u16,
            dwFileAttributes: u32,
            psfi: *mut SHFILEINFOW,
            cbFileInfo: u32,
            uFlags: u32,
        ) -> isize;
        fn DestroyIcon(hIcon: *mut std::ffi::c_void) -> i32;
        fn GetDC(hWnd: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
        fn ReleaseDC(hWnd: *mut std::ffi::c_void, hDC: *mut std::ffi::c_void) -> i32;
        fn CreateCompatibleDC(hdc: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
        fn DeleteDC(hdc: *mut std::ffi::c_void) -> i32;
        fn CreateDIBSection(
            hdc: *mut std::ffi::c_void,
            pbmi: *const std::ffi::c_void,
            usage: u32,
            ppvBits: *mut *mut u8,
            hSection: *mut std::ffi::c_void,
            offset: u32,
        ) -> *mut std::ffi::c_void;
        fn SelectObject(
            hdc: *mut std::ffi::c_void,
            h: *mut std::ffi::c_void,
        ) -> *mut std::ffi::c_void;
        fn DeleteObject(h: *mut std::ffi::c_void) -> i32;
        fn DrawIconEx(
            hdc: *mut std::ffi::c_void,
            xLeft: i32,
            yTop: i32,
            hIcon: *mut std::ffi::c_void,
            cxWidth: i32,
            cyWidth: i32,
            istepIfAniCur: u32,
            hbrFlickerFreeDraw: *mut std::ffi::c_void,
            diFlags: u32,
        ) -> i32;
    }

    let path_str = path.to_string_lossy().replace("/", "\\");
    let path_wide: Vec<u16> = OsStr::new(&path_str)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    unsafe {
        let mut shfi: SHFILEINFOW = std::mem::zeroed();
        let ret = SHGetFileInfoW(
            path_wide.as_ptr(),
            0,
            &mut shfi,
            std::mem::size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_LARGEICON | SHGFI_USEFILEATTRIBUTES,
        );
        if ret == 0 || shfi.hIcon.is_null() {
            return Err("SHGetFileInfoW failed".to_string());
        }

        let hicon = shfi.hIcon;
        let screen_dc = GetDC(std::ptr::null_mut());
        if screen_dc.is_null() {
            DestroyIcon(hicon);
            return Err("GetDC failed".to_string());
        }

        let mem_dc = CreateCompatibleDC(screen_dc);
        if mem_dc.is_null() {
            ReleaseDC(std::ptr::null_mut(), screen_dc);
            DestroyIcon(hicon);
            return Err("CreateCompatibleDC failed".to_string());
        }

        let w = ICON_SIZE;
        let h = ICON_SIZE;
        let bpp: u16 = 32;
        let row_bytes = ((w as u32 * bpp as u32 + 31) / 32) * 4;
        let pixel_size = row_bytes * h as u32;

        let bmi = BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: w,
            biHeight: -h,
            biPlanes: 1,
            biBitCount: bpp,
            biCompression: 0,
            biSizeImage: pixel_size,
            biXPelsPerMeter: 0,
            biYPelsPerMeter: 0,
            biClrUsed: 0,
            biClrImportant: 0,
        };

        let mut pixel_ptr: *mut u8 = std::ptr::null_mut();
        let hbmp = CreateDIBSection(
            mem_dc,
            &bmi as *const _ as *const std::ffi::c_void,
            0,
            &mut pixel_ptr,
            std::ptr::null_mut(),
            0,
        );
        if hbmp.is_null() || pixel_ptr.is_null() {
            DeleteDC(mem_dc);
            ReleaseDC(std::ptr::null_mut(), screen_dc);
            DestroyIcon(hicon);
            return Err("CreateDIBSection failed".to_string());
        }

        let old_bmp = SelectObject(mem_dc, hbmp);
        DrawIconEx(mem_dc, 0, 0, hicon, w, h, 0, std::ptr::null_mut(), 0x0003);
        SelectObject(mem_dc, old_bmp);

        // Read raw BGRA pixels, convert to RGBA
        let pixel_count = (row_bytes * h as u32) as usize;
        let bgra = std::slice::from_raw_parts(pixel_ptr, pixel_count);
        let mut rgba = Vec::with_capacity((w * h * 4) as usize);
        for y in 0..h {
            let row_start = (y as u32 * row_bytes) as usize;
            for x in 0..w {
                let i = row_start + (x * 4) as usize;
                rgba.push(bgra[i + 2]); // R
                rgba.push(bgra[i + 1]); // G
                rgba.push(bgra[i]); // B
                rgba.push(bgra[i + 3]); // A
            }
        }

        let mut png_bytes = Vec::new();
        PngEncoder::new(&mut png_bytes)
            .write_image(&rgba, w as u32, h as u32, ColorType::Rgba8.into())
            .map_err(|e| format!("PNG encode: {}", e))?;

        SelectObject(mem_dc, old_bmp);
        DeleteObject(hbmp);
        DeleteDC(mem_dc);
        ReleaseDC(std::ptr::null_mut(), screen_dc);
        DestroyIcon(hicon);

        Ok(png_bytes)
    }
}

// ── Windows clipboard (CF_HDROP) ──
fn write_windows_clipboard(paths: &[PathBuf]) {
    unsafe {
        extern "system" {
            fn OpenClipboard(h: *mut std::ffi::c_void) -> i32;
            fn EmptyClipboard() -> i32;
            fn SetClipboardData(f: u32, m: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
            fn CloseClipboard() -> i32;
            fn GlobalAlloc(f: u32, b: usize) -> *mut std::ffi::c_void;
            fn GlobalLock(m: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
            fn GlobalUnlock(m: *mut std::ffi::c_void) -> i32;
            fn GlobalFree(h: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
        }
        let mut retries = 3;
        while retries > 0 {
            if OpenClipboard(std::ptr::null_mut()) != 0 {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
            retries -= 1;
        }
        if retries == 0 {
            return;
        }
        EmptyClipboard();
        let mut wide: Vec<u16> = Vec::new();
        for p in paths {
            wide.extend(p.to_string_lossy().encode_utf16());
            wide.push(0);
        }
        wide.push(0);
        let hdr = std::mem::size_of::<u32>() * 5;
        let sz = hdr + wide.len() * 2;
        let h = GlobalAlloc(2, sz);
        if h.is_null() {
            CloseClipboard();
            return;
        }
        let p = GlobalLock(h) as *mut u8;
        std::ptr::write_bytes(p, 0, sz);
        (p as *mut u32).write(hdr as u32);
        (p.add(16) as *mut u32).write(1); // fWide = TRUE
        std::ptr::copy_nonoverlapping(wide.as_ptr(), p.add(hdr) as *mut u16, wide.len());
        GlobalUnlock(h);
        if SetClipboardData(15, h).is_null() {
            GlobalFree(h);
        }
        CloseClipboard();
    }
}

fn read_windows_clipboard() -> Option<Vec<PathBuf>> {
    unsafe {
        extern "system" {
            fn OpenClipboard(h: *mut std::ffi::c_void) -> i32;
            fn GetClipboardData(f: u32) -> *mut std::ffi::c_void;
            fn CloseClipboard() -> i32;
            fn IsClipboardFormatAvailable(f: u32) -> i32;
            fn DragQueryFileW(h: *mut std::ffi::c_void, i: u32, b: *mut u16, c: u32) -> u32;
        }
        let mut retries = 3;
        while retries > 0 {
            if OpenClipboard(std::ptr::null_mut()) != 0 {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
            retries -= 1;
        }
        if retries == 0 {
            return None;
        }
        if IsClipboardFormatAvailable(15) == 0 {
            CloseClipboard();
            return None;
        }
        let hd = GetClipboardData(15);
        if hd.is_null() {
            CloseClipboard();
            return None;
        }
        let cnt = DragQueryFileW(hd, 0xFFFFFFFF, std::ptr::null_mut(), 0);
        let mut v = Vec::with_capacity(cnt as usize);
        let mut b = vec![0u16; 32768];
        for i in 0..cnt {
            let l = DragQueryFileW(hd, i, b.as_mut_ptr(), b.len() as u32) as usize;
            if l > 0 {
                v.push(PathBuf::from(String::from_utf16_lossy(&b[..l])));
            }
        }
        CloseClipboard();
        Some(v)
    }
}

// ── Native drag (COM DoDragDrop, kept from original native_drag.rs) ──
fn native_drag_impl(paths: &[String]) -> Result<String, String> {
    if paths.is_empty() {
        return Err("No paths".into());
    }
    unsafe { do_drag_drop(paths) }
}

unsafe fn do_drag_drop(paths: &[String]) -> Result<String, String> {
    let mut wide: Vec<u16> = Vec::new();
    for p in paths {
        wide.extend(p.encode_utf16());
        wide.push(0);
    }
    wide.push(0);
    let hdr: u32 = 20;
    let total = hdr as usize + wide.len() * 2;

    extern "system" {
        fn OleInitialize(_: *mut std::ffi::c_void) -> i32;
        fn OleUninitialize();
        fn DoDragDrop(
            data: *mut std::ffi::c_void,
            src: *mut std::ffi::c_void,
            ok: u32,
            eff: *mut u32,
        ) -> i32;
        fn GlobalAlloc(f: u32, b: usize) -> *mut std::ffi::c_void;
        fn GlobalLock(h: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
        fn GlobalUnlock(h: *mut std::ffi::c_void) -> i32;
    }

    struct OleGuard;
    impl OleGuard {
        fn init() -> Result<Self, String> {
            if unsafe { OleInitialize(std::ptr::null_mut()) } >= 0 {
                Ok(OleGuard)
            } else {
                Err("OleInit failed".into())
            }
        }
    }
    impl Drop for OleGuard {
        fn drop(&mut self) {
            unsafe {
                OleUninitialize();
            }
        }
    }

    let _ole = OleGuard::init()?;

    let hmem = GlobalAlloc(2, total);
    if hmem.is_null() {
        return Err("Alloc failed".into());
    }
    let ptr = GlobalLock(hmem) as *mut u8;
    std::ptr::write_bytes(ptr, 0, total);
    std::ptr::write(ptr as *mut u32, hdr);
    std::ptr::write(ptr.add(16) as *mut u32, 1);
    std::ptr::copy_nonoverlapping(wide.as_ptr(), ptr.add(hdr as usize) as *mut u16, wide.len());
    GlobalUnlock(hmem);

    #[repr(C)]
    struct Obj {
        vtbl: *const *const std::ffi::c_void,
        refs: u32,
        hdrop: *mut std::ffi::c_void,
    }

    unsafe extern "system" fn qi(
        this: *mut std::ffi::c_void,
        _: *const std::ffi::c_void,
        ppv: *mut *mut std::ffi::c_void,
    ) -> i32 {
        *ppv = this;
        0
    }
    unsafe extern "system" fn addref(this: *mut std::ffi::c_void) -> u32 {
        let o = &mut *(this as *mut Obj);
        o.refs += 1;
        o.refs
    }
    unsafe extern "system" fn release(this: *mut std::ffi::c_void) -> u32 {
        let o = &mut *(this as *mut Obj);
        o.refs -= 1;
        o.refs
    }
    unsafe extern "system" fn get_data(
        this: *mut std::ffi::c_void,
        pfe: *mut std::ffi::c_void,
        psm: *mut std::ffi::c_void,
    ) -> i32 {
        if pfe.is_null() || psm.is_null() {
            return 0x80070057u32 as i32;
        }
        let cf = *(pfe as *const u32);
        let tymed = *(pfe.add(12) as *const u32);
        if cf != 15 || (tymed & 1) == 0 {
            return 0x80040064u32 as i32;
        }
        let o = &*(this as *const Obj);
        *(psm as *mut u32) = 1;
        *(psm.add(std::mem::size_of::<usize>()) as *mut *mut std::ffi::c_void) = o.hdrop;
        *(psm.add(std::mem::size_of::<usize>() * 2) as *mut *mut std::ffi::c_void) =
            std::ptr::null_mut();
        0
    }
    unsafe extern "system" fn qgd(_: *mut std::ffi::c_void, pfe: *mut std::ffi::c_void) -> i32 {
        if pfe.is_null() {
            return 0x80070057u32 as i32;
        }
        let cf = *(pfe as *const u32);
        let tymed = *(pfe.add(12) as *const u32);
        if cf == 15 && (tymed & 1) != 0 {
            0
        } else {
            0x80040064u32 as i32
        }
    }
    unsafe extern "system" fn notimpl2(_: *mut std::ffi::c_void, _: *mut std::ffi::c_void) -> i32 {
        0x80004001u32 as i32
    }
    unsafe extern "system" fn notimpl3(
        _: *mut std::ffi::c_void,
        _: *mut std::ffi::c_void,
        _: *mut std::ffi::c_void,
    ) -> i32 {
        0x80004001u32 as i32
    }
    unsafe extern "system" fn notimpl4(
        _: *mut std::ffi::c_void,
        _: *mut std::ffi::c_void,
        _: *mut std::ffi::c_void,
        _: i32,
    ) -> i32 {
        0x80004001u32 as i32
    }
    unsafe extern "system" fn notimpl5(_: *mut std::ffi::c_void, _: u32) -> i32 {
        0x80004001u32 as i32
    }
    unsafe extern "system" fn notimpl6(
        _: *mut std::ffi::c_void,
        _: u32,
        _: *mut *mut std::ffi::c_void,
    ) -> i32 {
        0x80004001u32 as i32
    }
    unsafe extern "system" fn notimpl7(
        _: *mut std::ffi::c_void,
        _: *mut std::ffi::c_void,
        _: u32,
        _: *mut std::ffi::c_void,
        _: *mut u32,
    ) -> i32 {
        0x80004001u32 as i32
    }
    unsafe extern "system" fn notimpl8(
        _: *mut std::ffi::c_void,
        _: *mut *mut std::ffi::c_void,
    ) -> i32 {
        0x80004001u32 as i32
    }

    let vtbl: [*const std::ffi::c_void; 12] = [
        qi as _,
        addref as _,
        release as _,
        get_data as _,
        notimpl3 as _,
        qgd as _,
        notimpl2 as _,
        notimpl4 as _,
        notimpl6 as _,
        notimpl7 as _,
        notimpl5 as _,
        notimpl8 as _,
    ];

    unsafe extern "system" fn ds_rel(_: *mut std::ffi::c_void) -> u32 {
        1
    }
    unsafe extern "system" fn ds_qc(_: *mut std::ffi::c_void, esc: i32, ks: u32) -> i32 {
        if esc != 0 {
            0x00040101u32 as i32
        } else if (ks & 3) == 0 {
            0x00040100u32 as i32
        } else {
            0
        }
    }
    let ds_vtbl: [*const std::ffi::c_void; 5] =
        [qi as _, ds_rel as _, ds_rel as _, ds_qc as _, ds_rel as _];

    let mut obj = Box::new(Obj {
        vtbl: vtbl.as_ptr(),
        refs: 1,
        hdrop: hmem,
    });
    let ds_ptr: *const *const std::ffi::c_void = ds_vtbl.as_ptr();

    let mut effect: u32 = 0;
    let hr = DoDragDrop(
        &mut *obj as *mut Obj as *mut std::ffi::c_void,
        std::mem::transmute(&ds_ptr),
        3,
        &mut effect,
    );

    if hr < 0 {
        return Err(format!("DDD:{hr}"));
    }
    Ok("done".into())
}

// ── Windows drive enumeration ──
fn get_windows_drives() -> Vec<DiskInfo> {
    use std::path::Path;
    let mut drives = Vec::new();

    for letter in b'A'..=b'Z' {
        let drive = format!("{}:\\", letter as char);
        let path = Path::new(&drive);
        if !path.exists() {
            continue;
        }

        let mut total: u64 = 0;
        let mut free: u64 = 0;

        let path_wide: Vec<u16> = drive.encode_utf16().chain(std::iter::once(0)).collect();

        unsafe {
            let ok = GetDiskFreeSpaceExW(
                path_wide.as_ptr(),
                std::ptr::null_mut(),
                &mut total,
                &mut free,
            );
            if ok == 0 {
                total = 0;
                free = 0;
            }
        }

        let label = get_windows_volume_label(&drive);
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
    drives
}

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

fn get_windows_volume_label(drive: &str) -> String {
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

fn get_windows_filesystem(drive: &str) -> String {
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
