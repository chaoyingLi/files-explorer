use tauri::{
    menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};

const SHOW_TEXT: &str = "  显示主窗口";
const HIDE_TEXT: &str = "  隐藏主窗口";

fn build_menu(app: &AppHandle, is_visible: bool) -> tauri::Result<tauri::menu::Menu<tauri::Wry>> {
    let toggle_text = if is_visible { HIDE_TEXT } else { SHOW_TEXT };
    let toggle = MenuItemBuilder::with_id("toggle", toggle_text).build(app)?;
    let downloads = MenuItemBuilder::with_id("downloads", "  下载").build(app)?;
    let documents = MenuItemBuilder::with_id("documents", "  文档").build(app)?;
    let settings = MenuItemBuilder::with_id("settings", "  设置…").build(app)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit = MenuItemBuilder::with_id("quit", "  退出").build(app)?;

    MenuBuilder::new(app)
        .item(&toggle)
        .item(&separator)
        .item(&downloads)
        .item(&documents)
        .item(&separator)
        .item(&settings)
        .item(&separator)
        .item(&quit)
        .build()
}

/// Rebuild tray menu when window visibility changes
pub fn rebuild_tray(app: &AppHandle, is_visible: bool) -> tauri::Result<()> {
    if let Some(tray) = app.tray_by_id("main-tray") {
        let menu = build_menu(app, is_visible)?;
        tray.set_menu(Some(menu))?;
    }
    Ok(())
}

pub fn create_tray(app: &AppHandle) -> tauri::Result<()> {
    let special = crate::drives::get_special_dirs().unwrap_or_default();
    let dirs = special;

    let menu = build_menu(app, true)?;

    let _tray = TrayIconBuilder::with_id("main-tray")
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .tooltip("Files Explorer")
        .on_menu_event(move |app, event| {
            let id = event.id().as_ref();
            let path: Option<String> = match id {
                "downloads" => Some(dirs.downloads.clone()),
                "documents" => Some(dirs.documents.clone()),
                "toggle" => {
                    if let Some(w) = app.get_webview_window("main") {
                        if w.is_visible().unwrap_or(false) {
                            let _ = w.hide();
                            let _ = rebuild_tray(app, false);
                        } else {
                            let _ = w.show();
                            let _ = w.set_focus();
                            let _ = rebuild_tray(app, true);
                        }
                    }
                    None
                }
                "settings" => {
                    let _ = app.emit("tray-open-settings", ());
                    if let Some(w) = app.get_webview_window("main") {
                        let _ = w.show();
                        let _ = w.set_focus();
                        let _ = rebuild_tray(app, true);
                    }
                    None
                }
                "quit" => {
                    let _ = app.emit("tray-quit", ());
                    let handle = app.clone();
                    std::thread::spawn(move || {
                        std::thread::sleep(std::time::Duration::from_millis(300));
                        handle.exit(0);
                    });
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
                        let _ = rebuild_tray(app, true);
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
                    if w.is_visible().unwrap_or(false) {
                        let _ = w.hide();
                        let _ = rebuild_tray(app, false);
                    } else {
                        let _ = w.show();
                        let _ = w.set_focus();
                        let _ = rebuild_tray(app, true);
                    }
                }
            }
        })
        .build(app)?;

    Ok(())
}
