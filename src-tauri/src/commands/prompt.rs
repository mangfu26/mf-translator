//! 提示词模板的 IPC 命令：查看、保存、重置。

use serde::Deserialize;
use tauri::State;

use crate::error::{AppError, AppResult};
use crate::prompt_store::{PromptKey, TemplateEntry};
use crate::state::{lock_poisoned, AppState};

fn parse_key(s: &str) -> AppResult<PromptKey> {
    match s {
        "system.general" => Ok(PromptKey::SystemGeneral),
        "system.academic" => Ok(PromptKey::SystemAcademic),
        "system.colloquial" => Ok(PromptKey::SystemColloquial),
        "system.polish" => Ok(PromptKey::SystemPolish),
        "user" => Ok(PromptKey::User),
        other => Err(AppError::Config(format!("未知的提示词模板键：{other}"))),
    }
}

/// 获取全部提示词模板（含用户编辑状态）。
#[tauri::command]
pub fn get_prompt_templates(state: State<'_, AppState>) -> AppResult<Vec<TemplateEntry>> {
    let store = state.prompt_store.lock().map_err(|_| lock_poisoned())?;
    store.all()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavePromptInput {
    pub key: String,
    pub content: String,
}

/// 保存单个提示词模板（标记为已修改）。
#[tauri::command]
pub fn save_prompt_template(
    input: SavePromptInput,
    state: State<'_, AppState>,
) -> AppResult<TemplateEntry> {
    let key = parse_key(&input.key)?;
    let content = input.content.trim().to_string();
    {
        let store = state.prompt_store.lock().map_err(|_| lock_poisoned())?;
        store.set_modified(key, &content)?;
    }
    Ok(TemplateEntry {
        key: key.as_str().to_string(),
        content,
        user_modified: true,
    })
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResetPromptInput {
    pub key: Option<String>,
}

/// 重置提示词模板：全部或单个，恢复内置默认值。
#[tauri::command]
pub fn reset_prompt_template(
    input: ResetPromptInput,
    state: State<'_, AppState>,
) -> AppResult<Vec<TemplateEntry>> {
    {
        let store = state.prompt_store.lock().map_err(|_| lock_poisoned())?;
        if let Some(key_str) = input.key.filter(|s| !s.trim().is_empty()) {
            let key = parse_key(&key_str)?;
            store.reset(key)?;
        } else {
            store.reset_all()?;
        }
    }
    let store = state.prompt_store.lock().map_err(|_| lock_poisoned())?;
    store.all()
}
