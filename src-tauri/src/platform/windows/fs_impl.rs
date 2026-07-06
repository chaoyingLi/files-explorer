// platform/windows/fs_impl.rs
// Windows file-system extension implementation.

use crate::core::error::AppError;
use crate::platform::PlatformFsExt;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

pub struct FsExtImpl;

impl PlatformFsExt for FsExtImpl {
    fn resolve_shortcut(&self, path: &Path) -> Option<PathBuf> {
        match path.extension().and_then(|e| e.to_str()) {
            Some(ext) if ext.eq_ignore_ascii_case("lnk") => resolve_lnk_via_com(path),
            _ => std::fs::read_link(path).ok(),
        }
    }

    fn is_hidden(&self, path: &Path) -> bool {
        use std::os::windows::fs::MetadataExt;
        if let Ok(meta) = path.metadata() {
            const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;
            if meta.file_attributes() & FILE_ATTRIBUTE_HIDDEN != 0 {
                return true;
            }
        }
        path.file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.starts_with('.'))
            .unwrap_or(false)
    }

    fn created_time(&self, metadata: &std::fs::Metadata) -> Option<SystemTime> {
        metadata.created().ok()
    }

    fn modified_time(&self, metadata: &std::fs::Metadata) -> Option<SystemTime> {
        metadata.modified().ok()
    }

    fn accessed_time(&self, metadata: &std::fs::Metadata) -> Option<SystemTime> {
        metadata.accessed().ok()
    }

    fn is_symlink(&self, path: &Path) -> bool {
        path.is_symlink()
            || path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.eq_ignore_ascii_case("lnk"))
                .unwrap_or(false)
    }

    fn path_eq(&self, a: &Path, b: &Path) -> bool {
        a.as_os_str().eq_ignore_ascii_case(b.as_os_str())
    }

    fn normalize_frontend_path(&self, path: &Path) -> String {
        path.to_string_lossy().replace('\\', "/")
    }

    fn check_writable(&self, path: &Path) -> Result<(), AppError> {
        use std::os::windows::fs::MetadataExt;
        if let Ok(meta) = path.metadata() {
            const FILE_ATTRIBUTE_READONLY: u32 = 0x1;
            if meta.file_attributes() & FILE_ATTRIBUTE_READONLY != 0 {
                return Err(AppError::PermissionDenied(
                    "File is read-only. Right-click → Properties → uncheck 'Read-only'.".into(),
                ));
            }
        }
        // For directories, test actual writability
        let target = if path.is_dir() {
            path.join(".write_test")
        } else {
            path.with_extension("write_test_tmp")
        };
        if let Err(_) = std::fs::write(&target, b"") {
            return Err(AppError::PermissionDenied(
                "No write permission. Check file attributes and running as administrator.".into(),
            ));
        }
        let _ = std::fs::remove_file(&target);
        Ok(())
    }
}

impl FsExtImpl {
    pub fn instance() -> &'static FsExtImpl {
        &FsExtImpl
    }
}

// ── .lnk resolution via IShellLinkW COM ──
fn resolve_lnk_via_com(lnk_path: &Path) -> Option<PathBuf> {
    // Convert path to wide string for COM
    let path_str = lnk_path.to_string_lossy().replace('/', "\\");
    let wide_path: Vec<u16> = path_str.encode_utf16().chain(std::iter::once(0)).collect();

    unsafe {
        use std::ffi::c_void;

        // COM CLSID and IID for IShellLinkW
        // CLSID_ShellLink = {00021401-0000-0000-C000-000000000046}
        // IID_IShellLinkW  = {000214F9-0000-0000-C000-000000000046}
        // IID_IPersistFile = {0000010B-0000-0000-C000-000000000046}
        // GUIDs as [u32; 4] (little-endian byte layout):
        // {00021401-0000-0000-C000-000000000046}
        let clsid_shelllink: [u32; 4] = [0x00021401, 0x0000, 0x000000C0, 0x46000000];
        // {000214F9-0000-0000-C000-000000000046}
        let iid_ishelllink: [u32; 4] = [0x000214F9, 0x0000, 0x000000C0, 0x46000000];
        // {0000010B-0000-0000-C000-000000000046}
        let iid_ipersistfile: [u32; 4] = [0x0000010B, 0x0000, 0x000000C0, 0x46000000];

        extern "system" {
            fn CoInitializeEx(reserved: *mut c_void, coinit: u32) -> i32;
            fn CoUninitialize();
            fn CoCreateInstance(
                clsid: *const [u32; 4],
                unk_outer: *mut c_void,
                cls_ctx: u32,
                iid: *const [u32; 4],
                ppv: *mut *mut c_void,
            ) -> i32;
        }

        // IShellLinkW vtable: 21 methods (IUnknown 3 + IShellLinkW 18)
        // We only need GetPath (index 3+3=6) and Resolve (index 3+8=11)
        // IPersistFile vtable: 8 methods (IUnknown 3 + IPersistFile 5)
        // We only need Load (index 3+4=7)

        const COINIT_APARTMENTTHREADED: u32 = 0x2;
        const CLSCTX_INPROC_SERVER: u32 = 0x1;

        let hr = CoInitializeEx(std::ptr::null_mut(), COINIT_APARTMENTTHREADED);
        if hr < 0 {
            // STILL_ACTIVE or already initialized — continue
        }

        let mut shell_link: *mut c_void = std::ptr::null_mut();
        let hr = CoCreateInstance(
            &clsid_shelllink,
            std::ptr::null_mut(),
            CLSCTX_INPROC_SERVER,
            &iid_ishelllink,
            &mut shell_link,
        );

        if hr < 0 || shell_link.is_null() {
            CoUninitialize();
            return None;
        }

        // QueryInterface for IPersistFile
        let mut persist_file: *mut c_void = std::ptr::null_mut();
        let vtable = *(shell_link as *const *const c_void);
        let qi: unsafe extern "system" fn(*mut c_void, *const [u32; 4], *mut *mut c_void) -> i32 =
            std::mem::transmute(*((vtable as *const *const c_void).add(0)));
        let hr = qi(shell_link, &iid_ipersistfile, &mut persist_file);

        if hr < 0 || persist_file.is_null() {
            // Release IShellLinkW (IUnknown::Release = vtable[2])
            let vtable = *(shell_link as *const *const c_void);
            let release: unsafe extern "system" fn(*mut c_void) -> u32 =
                std::mem::transmute(*((vtable as *const *const c_void).add(2)));
            release(shell_link);
            CoUninitialize();
            return None;
        }

        // IPersistFile::Load = vtable[5 + 2] = vtable[7]?
        // Actually IPersistFile vtable layout:
        // 0: QueryInterface, 1: AddRef, 2: Release (IUnknown)
        // 3: GetClassID (IPersist)
        // 4: IsDirty, 5: Load, 6: Save, 7: SaveCompleted, 8: GetCurFile
        // So Load is vtable[5]
        let pf_vtable = *(persist_file as *const *const c_void);
        let load: unsafe extern "system" fn(*mut c_void, *const u16, u32) -> i32 =
            std::mem::transmute(*((pf_vtable as *const *const c_void).add(5)));

        let hr = load(persist_file, wide_path.as_ptr(), 0);

        if hr < 0 {
            // Release IPersistFile + IShellLinkW
            let pf_release: unsafe extern "system" fn(*mut c_void) -> u32 =
                std::mem::transmute(*((pf_vtable as *const *const c_void).add(2)));
            pf_release(persist_file);
            let sl_vtable = *(shell_link as *const *const c_void);
            let sl_release: unsafe extern "system" fn(*mut c_void) -> u32 =
                std::mem::transmute(*((sl_vtable as *const *const c_void).add(2)));
            sl_release(shell_link);
            CoUninitialize();
            return None;
        }

        // IShellLinkW::GetPath = vtable[3 + 3] = vtable[6]
        // IShellLinkW::GetPath(szFile, cch, pfd, fFlags)
        let sl_vtable = *(shell_link as *const *const c_void);
        let get_path: unsafe extern "system" fn(
            *mut c_void,
            *mut u16,
            i32,
            *mut c_void,
            u32,
        ) -> i32 = std::mem::transmute(*((sl_vtable as *const *const c_void).add(6)));

        let mut buf = vec![0u16; 260]; // MAX_PATH
        let hr = get_path(
            shell_link,
            buf.as_mut_ptr(),
            buf.len() as i32,
            std::ptr::null_mut(),
            0, // SLGP_RAWPATH = 0x4, but 0 = default
        );

        // Release both COM objects
        let pf_release: unsafe extern "system" fn(*mut c_void) -> u32 =
            std::mem::transmute(*((pf_vtable as *const *const c_void).add(2)));
        pf_release(persist_file);
        let sl_release: unsafe extern "system" fn(*mut c_void) -> u32 =
            std::mem::transmute(*((sl_vtable as *const *const c_void).add(2)));
        sl_release(shell_link);
        CoUninitialize();

        if hr < 0 {
            return None;
        }

        let end = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
        let target = String::from_utf16_lossy(&buf[..end]);
        if target.is_empty() {
            None
        } else {
            Some(PathBuf::from(target))
        }
    }
}
