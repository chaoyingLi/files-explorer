// Native drag OUT — COM DoDragDrop with CF_HDROP
// External apps (Explorer, WeChat) receive native CF_HDROP file paths

#[cfg(not(target_os = "windows"))]
pub fn start_native_drag(_paths: &[String]) -> Result<String, String> {
    Err("Native drag only supported on Windows".into())
}

#[cfg(target_os = "windows")]
pub fn start_native_drag(paths: &[String]) -> Result<String, String> {
    log::info!("start_native_drag: {:?}", paths);
    if paths.is_empty() {
        return Err("No paths".into());
    }
    unsafe { do_drag_drop(paths) }
}

#[cfg(target_os = "windows")]
unsafe fn do_drag_drop(paths: &[String]) -> Result<String, String> {
    // Build DROPFILES
    let mut wide: Vec<u16> = Vec::new();
    for p in paths {
        wide.extend(p.encode_utf16());
        wide.push(0);
    }
    wide.push(0);
    let hdr: u32 = 20;
    let total = hdr as usize + wide.len() * 2;

    // Win32 imports
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

    // RAII guard for OleInitialize/OleUninitialize
    struct OleGuard;
    impl OleGuard {
        fn init() -> Result<Self, String> {
            unsafe {
                if OleInitialize(std::ptr::null_mut()) >= 0 {
                    Ok(OleGuard)
                } else {
                    Err("OleInit failed".into())
                }
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

    // COM struct: [vtbl_ptr, refs, hdrop]
    #[repr(C)]
    struct Obj {
        vtbl: *const *const std::ffi::c_void,
        refs: u32,
        hdrop: *mut std::ffi::c_void,
    }

    // IUnknown
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

    // IDataObject (GetData only, others return E_NOTIMPL)
    unsafe extern "system" fn get_data(
        this: *mut std::ffi::c_void,
        pfe: *mut std::ffi::c_void,
        psm: *mut std::ffi::c_void,
    ) -> i32 {
        if pfe.is_null() || psm.is_null() {
            return 0x80070057u32 as i32;
        }
        // FORMATETC: cf_format at offset 0 (u32), tymed at offset 12 (u32)
        let cf = *(pfe as *const u32);
        let tymed = *(pfe.add(12) as *const u32);
        if cf != 15 || (tymed & 1) == 0 {
            return 0x80040064u32 as i32;
        }
        let o = &*(this as *const Obj);
        // STGMEDIUM: tymed at 0 (u32), u at (pointer size)
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

    // IDropSource (5 methods)
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

    log::info!("DoDragDrop result: {}, effect: {}", hr, effect);
    // S_OK, DRAGDROP_S_DROP (0x00040100), and DRAGDROP_S_CANCEL are all success
    if hr < 0 {
        return Err(format!("DDD:{hr}"));
    }
    Ok("done".into())
}
