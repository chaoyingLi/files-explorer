// platform/mod.rs
// ══════════════════════════════════════════════════════════════
// ALL #[cfg(target_os)] converges HERE ONLY.
// Business modules (core/, commands/, utils/) MUST NEVER use
// #[cfg(target_os)] — import from platform::* instead.
// ══════════════════════════════════════════════════════════════

#[cfg(all(unix, not(target_os = "macos")))]
pub(crate) mod linux;
#[cfg(target_os = "macos")]
pub(crate) mod macos;
#[cfg(target_os = "windows")]
pub(crate) mod windows;

mod fs_ext;
mod path;
mod system;
mod window;

pub use fs_ext::PlatformFsExt;
pub use path::PlatformPath;
pub use system::PlatformSystem;
pub use window::PlatformWindow;

#[cfg(all(unix, not(target_os = "macos")))]
pub(crate) use linux as current;
#[cfg(target_os = "macos")]
pub(crate) use macos as current;
#[cfg(target_os = "windows")]
pub(crate) use windows as current;

/// Obtain the platform path provider (singleton).
pub fn path_provider() -> &'static dyn PlatformPath {
    current::PathImpl::instance()
}

/// Obtain the platform system provider (singleton).
pub fn system_provider() -> &'static dyn PlatformSystem {
    current::SystemImpl::instance()
}

/// Obtain the platform FS extension provider (singleton).
pub fn fs_ext_provider() -> &'static dyn PlatformFsExt {
    current::FsExtImpl::instance()
}

/// Obtain the platform window provider (singleton).
pub fn window_provider() -> &'static dyn PlatformWindow {
    current::WindowImpl::instance()
}
