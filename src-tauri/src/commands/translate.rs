use tauri::ipc::Channel;
use tauri::State;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::config::load_api_key;
use crate::error::{AppError, AppResult};
use crate::provider::{TranslateRequest, TranslationEvent};
use crate::state::{lock_poisoned, AppState};
use crate::translation;

/// 发起一次流式翻译。返回任务 id（用于取消）。
/// 事件经由 `channel` 推送：Delta / Finished / Failed。
#[tauri::command]
pub async fn translate_text(
    request: TranslateRequest,
    channel: Channel<TranslationEvent>,
    state: State<'_, AppState>,
) -> AppResult<Uuid> {
    let provider = {
        let config = state.config.lock().map_err(|_| lock_poisoned())?;
        config
            .active_provider
            .clone()
            .ok_or_else(|| AppError::Config("尚未配置供应商，请先在设置中完成配置".to_string()))?
    };
    let api_key = load_api_key(&provider)?;

    let id = Uuid::new_v4();
    let token = CancellationToken::new();
    state.tasks.insert(id, token.clone());

    let client = state.client.clone();
    let history = state.history.clone();
    let tasks = state.tasks.clone();

    tauri::async_runtime::spawn(async move {
        let _ = translation::run(client, provider, api_key, request, channel, token, history).await;
        tasks.remove(&id);
    });
    Ok(id)
}

#[tauri::command]
pub async fn cancel_translation(task_id: String, state: State<'_, AppState>) -> AppResult<bool> {
    let id =
        Uuid::parse_str(&task_id).map_err(|_| AppError::Config("无效的任务 id".to_string()))?;
    Ok(state.tasks.cancel(&id))
}
