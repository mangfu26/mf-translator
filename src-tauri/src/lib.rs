pub mod commands;
pub mod config;
pub mod error;
pub mod history;
pub mod prompt_store;
pub mod provider;
pub mod state;
pub mod translation;

use std::sync::{Arc, Mutex};

use tauri::menu::{Menu, MenuItem};
use tauri::tray::{TrayIconBuilder, TrayIconEvent};
use tauri::{Manager, WindowEvent};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

use crate::config::ConfigStore;
use crate::state::AppState;

/// 默认全局快捷键：Alt+Shift+T（后续从配置读取自定义值）
const DEFAULT_QUICK_SHORTCUT: &str = "Alt+Shift+T";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        toggle_quick_window(app, shortcut);
                    }
                })
                .build(),
        )
        .setup(|app| {
            let data_dir = app.path().app_data_dir().expect("无法解析应用数据目录");
            std::fs::create_dir_all(&data_dir)?;

            let config_store = ConfigStore::new(data_dir.join("config.json"));
            let config = config_store.load();
            let db = history::HistoryDb::open(&data_dir.join("history.db"))?;
            let prompts = prompt_store::PromptStore::open(&data_dir.join("prompts.db"))?;
            // Gitee raw 端点返回 302 重定向到实际 CDN 地址，须允许跟随；限制次数防循环。
            let client = reqwest::Client::builder()
                .redirect(reqwest::redirect::Policy::limited(5))
                .connect_timeout(std::time::Duration::from_secs(10))
                .read_timeout(std::time::Duration::from_secs(60))
                .build()?;

            app.manage(AppState {
                client,
                config: Arc::new(Mutex::new(config)),
                config_store: Arc::new(config_store),
                tasks: Arc::new(translation::TaskRegistry::default()),
                history: Arc::new(Mutex::new(db)),
                prompt_store: Arc::new(Mutex::new(prompts)),
            });

            // 注册全局快捷键（Alt+Shift+T）——后续改为读配置自定义值
            let shortcut: Shortcut = DEFAULT_QUICK_SHORTCUT.parse().expect("默认快捷键解析失败");
            app.global_shortcut().register(shortcut)?;

            setup_tray(app)?;
            setup_main_window(app)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::health::health_check,
            commands::translate::translate_text,
            commands::translate::cancel_translation,
            commands::settings::list_presets,
            commands::settings::get_provider_config,
            commands::settings::save_provider_config,
            commands::settings::test_connection,
            commands::history::list_history,
            commands::history::delete_history_item,
            commands::history::clear_history,
            commands::update::check_update,
            commands::window::get_window_state,
            commands::window::set_always_on_top,
            commands::window::hide_quick_window,
            commands::window::toggle_always_on_top,
            commands::prompt::get_prompt_templates,
            commands::prompt::save_prompt_template,
            commands::prompt::reset_prompt_template,
        ])
        .run(tauri::generate_context!())
        .expect("Tauri 应用启动失败");
}

fn setup_tray(app: &mut tauri::App) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "toggle", "显示 / 隐藏", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &quit])?;

    TrayIconBuilder::with_id("main-tray")
        .icon(app.default_window_icon().expect("缺少应用图标").clone())
        .tooltip("MF 翻译")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "toggle" => toggle_main_window(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if matches!(event, TrayIconEvent::DoubleClick { .. }) {
                toggle_main_window(tray.app_handle());
            }
        })
        .build(app)?;
    Ok(())
}

fn toggle_main_window<M: Manager<tauri::Wry>>(manager: &M) {
    if let Some(window) = manager.get_webview_window("main") {
        let visible = window.is_visible().unwrap_or(false);
        let focused = window.is_focused().unwrap_or(false);
        if visible && focused {
            let _ = window.hide();
        } else {
            let _ = window.show();
            let _ = window.unminimize();
            let _ = window.set_focus();
        }
    }
}

fn setup_main_window(app: &mut tauri::App) -> tauri::Result<()> {
    let window = app.get_webview_window("main").expect("主窗口缺失");
    // 关闭窗口 = 隐藏到托盘；真正退出走托盘菜单的“退出”
    let handle = window.clone();
    window.on_window_event(move |event| {
        if let WindowEvent::CloseRequested { api, .. } = event {
            api.prevent_close();
            let _ = handle.hide();
        }
    });
    Ok(())
}

/// 全局快捷键回调：检测到快捷小窗快捷键时，切换其显示/隐藏。
fn toggle_quick_window<M: Manager<tauri::Wry>>(manager: &M, _shortcut: &Shortcut) {
    if let Some(window) = manager.get_webview_window("quick") {
        let visible = window.is_visible().unwrap_or(false);
        if visible {
            let _ = window.hide();
        } else {
            let _ = window.show();
            let _ = window.unminimize();
            let _ = window.set_focus();
        }
    }
}
