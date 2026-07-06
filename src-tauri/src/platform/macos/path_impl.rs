// platform/macos/path_impl.rs
// macOS path provider implementation.

use crate::platform::PlatformPath;
use std::path::PathBuf;

pub struct PathImpl;

impl PlatformPath for PathImpl {
    fn app_data_dir(&self) -> PathBuf {
        home()
            .join("Library")
            .join("Application Support")
            .join("files-explorer")
    }

    fn app_cache_dir(&self) -> PathBuf {
        home().join("Library").join("Caches").join("files-explorer")
    }

    fn app_config_dir(&self) -> PathBuf {
        home()
            .join("Library")
            .join("Application Support")
            .join("files-explorer")
            .join("config")
    }

    fn app_log_dir(&self) -> PathBuf {
        home().join("Library").join("Logs").join("files-explorer")
    }

    fn app_resource_dir(&self) -> PathBuf {
        // macOS .app bundle: Contents/Resources/
        std::env::current_exe()
            .ok()
            .and_then(|p| {
                p.parent() // MacOS/
                    .and_then(|p| p.parent()) // Contents/
                    .map(|p| p.join("Resources"))
            })
            .unwrap_or_else(|| PathBuf::from("."))
    }

    fn home_dir(&self) -> PathBuf {
        home()
    }

    fn desktop_dir(&self) -> PathBuf {
        dirs::desktop_dir().unwrap_or_else(|| home().join("Desktop"))
    }

    fn documents_dir(&self) -> PathBuf {
        dirs::document_dir().unwrap_or_else(|| home().join("Documents"))
    }

    fn downloads_dir(&self) -> PathBuf {
        dirs::download_dir().unwrap_or_else(|| home().join("Downloads"))
    }

    fn pictures_dir(&self) -> PathBuf {
        dirs::picture_dir().unwrap_or_else(|| home().join("Pictures"))
    }

    fn music_dir(&self) -> PathBuf {
        dirs::audio_dir().unwrap_or_else(|| home().join("Music"))
    }

    fn videos_dir(&self) -> PathBuf {
        dirs::video_dir().unwrap_or_else(|| home().join("Movies"))
    }

    fn temp_dir(&self) -> PathBuf {
        dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("files-explorer-preview")
    }
}

fn home() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/"))
}

impl PathImpl {
    pub fn instance() -> &'static PathImpl {
        &PathImpl
    }
}
