//! 翻译任务编排：驱动协议流、把事件转发到前端、完成后落库、响应取消。

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tauri::ipc::Channel;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::config::ProviderConfig;
use crate::error::{AppError, AppResult};
use crate::history::{HistoryDb, HistoryInsert};
use crate::prompt_store::PromptStore;
use crate::provider::{protocol_for, PromptPair, TranslateRequest, TranslationEvent};
use crate::translation::prompt;

/// 运行中的翻译任务表：任务 id → 取消令牌。
#[derive(Default)]
pub struct TaskRegistry {
    inner: Mutex<HashMap<Uuid, CancellationToken>>,
}

impl TaskRegistry {
    pub fn insert(&self, id: Uuid, token: CancellationToken) {
        if let Ok(mut guard) = self.inner.lock() {
            guard.insert(id, token);
        }
    }

    pub fn remove(&self, id: &Uuid) {
        if let Ok(mut guard) = self.inner.lock() {
            guard.remove(id);
        }
    }

    pub fn cancel(&self, id: &Uuid) -> bool {
        match self.inner.lock() {
            Ok(guard) => guard.get(id).is_some_and(|token| {
                token.cancel();
                true
            }),
            Err(_) => false,
        }
    }
}

/// 执行一次翻译：把协议事件流转发到 channel，完成（或失败/取消）后写历史。
#[allow(clippy::too_many_arguments)] // 编排层需持有客户端/供应商/密钥/请求/信道/取消/历史/提示词库
pub async fn run(
    client: reqwest::Client,
    provider: ProviderConfig,
    api_key: String,
    request: TranslateRequest,
    channel: Channel<TranslationEvent>,
    cancel: CancellationToken,
    history: Arc<Mutex<HistoryDb>>,
    prompt_store: Arc<Mutex<PromptStore>>,
) -> AppResult<()> {
    let protocol = protocol_for(provider.protocol);
    // 从模板动态生成提示词（用户改过则用用户版，否则默认）
    let prompts: PromptPair = {
        let store = prompt_store
            .lock()
            .map_err(|_| AppError::Config("提示词存储异常".into()))?;
        prompt::build_prompt_pair(&store, &request)?
    };
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<TranslationEvent>();

    let worker = tauri::async_runtime::spawn({
        let cancel = cancel.child_token();
        let provider = provider.clone();
        let request = request.clone();
        let prompts = prompts.clone();
        async move {
            protocol
                .stream(
                    &client,
                    &provider.base_url,
                    &api_key,
                    &request,
                    &prompts,
                    cancel,
                    tx,
                )
                .await
        }
    });

    let mut translated = String::new();

    let terminal: TranslationEvent = loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                break TranslationEvent::Failed { error: AppError::Cancelled };
            }
            event = rx.recv() => match event {
                Some(TranslationEvent::Delta { text }) => {
                    translated.push_str(&text);
                    let _ = channel.send(TranslationEvent::Delta { text });
                }
                Some(ev @ (TranslationEvent::Finished { .. } | TranslationEvent::Failed { .. })) => {
                    let _ = channel.send(ev.clone());
                    break ev;
                }
                None => break TranslationEvent::Finished { usage: None },
            },
        }
    };
    worker.abort(); // 兜底：若协议任务仍在收尾，立即终止

    if let TranslationEvent::Finished { usage } = &terminal {
        if !translated.is_empty() {
            let row = HistoryInsert {
                source_text: &request.source_text,
                translated_text: &translated,
                source_language: request.source_language.as_deref(),
                target_language: &request.target_language,
                provider: &provider.name,
                model: &request.model,
                usage: *usage,
            };
            if let Ok(db) = history.lock() {
                let _ = db.insert(&row);
            }
        }
    }
    Ok(())
}
