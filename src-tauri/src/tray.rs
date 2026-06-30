use tauri::{
    menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};

pub fn create_tray(app: &AppHandle) -> tauri::Result<()> {
    let special = crate::drives::get_special_dirs().unwrap_or_default();

    let current = MenuItemBuilder::with_id("current_path", "📂 ...")
        .enabled(false)
        .build(app)?;

    let desktop = MenuItemBuilder::with_id("desktop", "🖥️ 桌面").build(app)?;
    let downloads = MenuItemBuilder::with_id("downloads", "📥 下载").build(app)?;
    let documents = MenuItemBuilder::with_id("documents", "📄 文档").build(app)?;
    let pictures = MenuItemBuilder::with_id("pictures", "🖼️ 图片").build(app)?;
    let music = MenuItemBuilder::with_id("music", "🎵 音乐").build(app)?;
    let videos = MenuItemBuilder::with_id("videos", "🎬 视频").build(app)?;

    let settings = MenuItemBuilder::with_id("settings", "⚙ 设置…").build(app)?;
    let show = MenuItemBuilder::with_id("show", "🔲 显示主窗口").build(app)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit = MenuItemBuilder::with_id("quit", "✕ 退出 Files Explorer").build(app)?;

    let menu = MenuBuilder::new(app)
        .item(&current)
        .item(&separator)
        .item(&desktop)
        .item(&downloads)
        .item(&documents)
        .item(&pictures)
        .item(&music)
        .item(&videos)
        .item(&separator)
        .item(&settings)
        .item(&show)
        .item(&separator)
        .item(&quit)
        .build()?;

    // Store dirs for menu event handler
    let dirs = special;

    let _tray = TrayIconBuilder::with_id("main-tray")
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .tooltip("Files Explorer")
        .on_menu_event(move |app, event| {
            let id = event.id().as_ref();
            let path: Option<String> = match id {
                "desktop" => Some(dirs.desktop.clone()),
                "downloads" => Some(dirs.downloads.clone()),
                "documents" => Some(dirs.documents.clone()),
                "pictures" => Some(dirs.pictures.clone()),
                "music" => Some(dirs.music.clone()),
                "videos" => Some(dirs.videos.clone()),
                "show" => {
                    if let Some(w) = app.get_webview_window("main") {
                        let _ = w.show();
                        let _ = w.set_focus();
                    }
                    None
                }
                "settings" => {
                    // Emit event to frontend to open settings
                    let _ = app.emit("tray-open-settings", ());
                    if let Some(w) = app.get_webview_window("main") {
                        let _ = w.show();
                        let _ = w.set_focus();
                    }
                    None
                }
                "quit" => {
                    app.exit(0);
                    #[allow(unreachable_code)]
                    None
                }
                _ => None,
            };

            if let Some(p) = path {
                if !p.is_empty() {
                    let _ = app.emit("tray-navigate", p);
                    if let Some(w) = app.get_webview_window("main") {
                        let _ = w.show();
                        let _ = w.set_focus();
                    }
                }
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.show();
                    let _ = w.set_focus();
                }
            }
        })
        .build(app)?;

    Ok(())
}
