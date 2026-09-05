//! 窗口控制命令：置顶切换、快捷小窗隐藏等。

use serde::{Deserialize, Serialize};
use tauri::{Manager, State, Window};

use crate::error::{AppError, AppResult};
use crate::state::AppState;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowState {
    pub label: String,
    pub visible: bool,
    pub always_on_top: bool,
}

fn window_by_label(window: &Window, label: &str) -> AppResult<tauri::WebviewWindow> {
    window
        .app_handle()
        .get_webview_window(label)
        .ok_or_else(|| AppError::Config(format!("窗口 {label} 不存在")))
}

/// 查询指定窗口的状态（可见性 + 是否置顶）。
#[tauri::command]
pub fn get_window_state(window: Window, label: String) -> AppResult<WindowState> {
    let w = window_by_label(&window, &label)?;
    Ok(WindowState {
        visible: w.is_visible().unwrap_or(false),
        always_on_top: w.is_always_on_top().unwrap_or(false),
        label,
    })
}

/// 设置指定窗口是否置顶。
#[tauri::command]
pub fn set_always_on_top(window: Window, label: String, value: bool) -> AppResult<()> {
    let w = window_by_label(&window, &label)?;
    w.set_always_on_top(value)
        .map_err(|e| AppError::Config(format!("设置置顶失败：{e}")))
}

/// 隐藏快捷小窗（Esc / 失焦调用）。
#[tauri::command]
pub fn hide_quick_window(window: Window, _state: State<'_, AppState>) -> AppResult<()> {
    if let Some(w) = window.app_handle().get_webview_window("quick") {
        let _ = w.hide();
    }
    Ok(())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToggleAlwaysOnTopInput {
    pub label: String,
    pub value: bool,
}

/// 从前端切换置顶（label 可为 main 或 quick）。
#[tauri::command]
pub fn toggle_always_on_top(window: Window, input: ToggleAlwaysOnTopInput) -> AppResult<()> {
    let w = window_by_label(&window, &input.label)?;
    w.set_always_on_top(input.value)
        .map_err(|e| AppError::Config(format!("设置置顶失败：{e}")))
}
