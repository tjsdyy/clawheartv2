//! 系统托盘 — tauri 2 tray-icon
//!
//! 三色图标：
//!   - 绿（正常）— 默认
//!   - 黄（警告）— 有未读拦截或公告
//!   - 红（关键）— Kill Switch 激活 或 关键拦截
//!
//! 左键 → 弹 mini UI（应用内 TrayPopup 组件）
//! 右键 → 系统菜单：暂停防护 / Kill Switch / 打开主面板 / 退出

use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Runtime};

#[derive(Debug, Clone, Copy)]
pub enum TrayColor {
    Green,
    Yellow,
    Red,
}

impl TrayColor {
    pub fn icon_path(&self) -> &'static str {
        match self {
            TrayColor::Green => "icons/tray-green.png",
            TrayColor::Yellow => "icons/tray-yellow.png",
            TrayColor::Red => "icons/tray-red.png",
        }
    }
}

pub fn install<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let pause = MenuItem::with_id(app, "pause", "暂停防护 5 分钟", true, None::<&str>)?;
    let kill = MenuItem::with_id(app, "kill", "🚨 Kill Switch", true, None::<&str>)?;
    let show = MenuItem::with_id(app, "show", "▢ 打开主面板", true, None::<&str>)?;
    let separator = tauri::menu::PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "⤴ 退出", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&pause, &kill, &show, &separator, &quit])?;

    TrayIconBuilder::with_id("clawheart-tray")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .tooltip("ClawHeart · 防护中")
        .on_menu_event(|app, event| match event.id.as_ref() {
            "pause" => {
                tracing::info!("Tray: pause 5min");
                // W4: pause proxy via state
            }
            "kill" => {
                tracing::warn!("Tray: kill switch activate");
                if let Some(state) = app.try_state::<crate::state::AppState>() {
                    state.kill_switch.activate_api();
                }
            }
            "show" => {
                if let Some(win) = app.get_webview_window("main") {
                    let _ = win.show();
                    let _ = win.set_focus();
                }
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button: MouseButton::Left, .. } = event {
                let app = tray.app_handle();
                if let Some(win) = app.get_webview_window("main") {
                    let _ = win.show();
                    let _ = win.set_focus();
                }
            }
        })
        .build(app)?;

    Ok(())
}

/// 更新托盘图标颜色（响应 kill_switch / 关键拦截 / 公告 状态变化）。
/// W4 起在 worker 中定期调用。
pub fn update_color<R: Runtime>(_app: &AppHandle<R>, _color: TrayColor) -> tauri::Result<()> {
    // W4: app.tray_by_id("clawheart-tray").set_icon(...)
    Ok(())
}
