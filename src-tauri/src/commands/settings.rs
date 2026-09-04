use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::State;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::config::{self, ProviderConfig};
use crate::error::{AppError, AppResult};
use crate::provider::presets::ProviderPreset;
use crate::provider::{protocol_for, ProtocolKind, TranslateRequest, TranslationEvent};
use crate::state::{lock_poisoned, AppState};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderConfigView {
    pub id: String,
    pub name: String,
    pub base_url: String,
    pub model: String,
    pub protocol: ProtocolKind,
    pub has_api_key: bool,
}

fn view_of(config: &ProviderConfig) -> ProviderConfigView {
    ProviderConfigView {
        id: config.id.clone(),
        name: config.name.clone(),
        base_url: config.base_url.clone(),
        model: config.model.clone(),
        protocol: config.protocol,
        has_api_key: config.api_key_ref.is_some(),
    }
}

#[tauri::command]
pub fn list_presets() -> AppResult<Vec<ProviderPreset>> {
    Ok(crate::provider::presets::builtin())
}

#[tauri::command]
pub fn get_provider_config(state: State<'_, AppState>) -> AppResult<Option<ProviderConfigView>> {
    let config = state.config.lock().map_err(|_| lock_poisoned())?;
    Ok(config.active_provider.as_ref().map(view_of))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderConfigInput {
    pub id: Option<String>,
    pub name: String,
    pub base_url: String,
    pub model: String,
    pub protocol: ProtocolKind,
}

#[tauri::command]
pub fn save_provider_config(
    input: ProviderConfigInput,
    api_key: Option<String>,
    state: State<'_, AppState>,
) -> AppResult<ProviderConfigView> {
    let base_url = input.base_url.trim().trim_end_matches('/').to_string();
    if !base_url.starts_with("http://") && !base_url.starts_with("https://") {
        return Err(AppError::Config(
            "API 地址必须以 http:// 或 https:// 开头".to_string(),
        ));
    }
    if input.model.trim().is_empty() {
        return Err(AppError::Config("请填写模型名称".to_string()));
    }

    let id = input
        .id
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    let mut provider = ProviderConfig {
        id: id.clone(),
        name: input.name.trim().to_string(),
        base_url,
        model: input.model.trim().to_string(),
        protocol: input.protocol,
        api_key_ref: None,
    };

    // 保留既有 Key 引用（避免未改动 Key 的保存动作清空凭据）
    {
        let config = state.config.lock().map_err(|_| lock_poisoned())?;
        if let Some(existing) = &config.active_provider {
            if existing.id == provider.id {
                provider.api_key_ref = existing.api_key_ref.clone();
            }
        }
    }

    match api_key.as_deref().map(str::trim) {
        Some(key) if !key.is_empty() => {
            config::store_api_key(&provider.id, key)?;
            provider.api_key_ref = Some(provider.id.clone());
        }
        Some(_) => {
            config::delete_api_key(&provider.id)?;
            provider.api_key_ref = None;
        }
        None => {}
    }

    let view = view_of(&provider);
    {
        let mut config = state.config.lock().map_err(|_| lock_poisoned())?;
        config.active_provider = Some(provider);
        state.config_store.save(&config)?;
    }
    Ok(view)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestConnectionInput {
    pub provider_id: Option<String>,
    pub base_url: String,
    pub model: String,
    pub protocol: ProtocolKind,
    /// None/空 表示尝试使用已保存的 Key。
    pub api_key: Option<String>,
}

/// 连接测试：用最小真实请求验证 地址 + Key + 协议 三者匹配。
#[tauri::command]
pub async fn test_connection(
    input: TestConnectionInput,
    state: State<'_, AppState>,
) -> AppResult<String> {
    let api_key = match input.api_key.as_deref().map(str::trim) {
        Some(key) if !key.is_empty() => key.to_string(),
        _ => {
            let config = state.config.lock().map_err(|_| lock_poisoned())?;
            match config
                .active_provider
                .as_ref()
                .filter(|p| Some(&p.id) == input.provider_id.as_ref())
            {
                Some(p) if p.api_key_ref.is_some() => config::load_api_key(p)?,
                _ => String::new(),
            }
        }
    };

    let TestConnectionInput {
        provider_id: _,
        base_url,
        model,
        protocol,
        api_key: _,
    } = input;

    let request = TranslateRequest {
        model,
        source_text: "Hello, world!".to_string(),
        target_language: "简体中文".to_string(),
        source_language: None,
    };

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<TranslationEvent>();
    let protocol_impl = protocol_for(protocol);
    let cancel = CancellationToken::new();
    let client = state.client.clone();

    let worker = tauri::async_runtime::spawn(async move {
        protocol_impl
            .stream(&client, &base_url, &api_key, &request, cancel, tx)
            .await
    });

    let outcome = tokio::time::timeout(Duration::from_secs(20), async {
        match rx.recv().await {
            Some(TranslationEvent::Delta { .. }) => Ok("连接成功，模型已正常响应".to_string()),
            Some(TranslationEvent::Finished { .. }) => Ok("连接成功".to_string()),
            Some(TranslationEvent::Failed { error }) => Err(error),
            None => Ok("连接成功".to_string()),
        }
    })
    .await;

    worker.abort();
    match outcome {
        Ok(result) => result,
        Err(_) => Err(AppError::Network("连接超时（20 秒无响应）".to_string())),
    }
}
