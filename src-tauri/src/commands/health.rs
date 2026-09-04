use crate::error::AppResult;
use crate::provider::ProtocolKind;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProtocolEntry {
    pub id: ProtocolKind,
    pub label: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthReport {
    pub app_version: String,
    pub engine_ready: bool,
    pub protocols: Vec<ProtocolEntry>,
}

/// IPC 链路与引擎自检入口。M2 起将纳入当前供应商连通状态。
#[tauri::command]
pub async fn health_check() -> AppResult<HealthReport> {
    Ok(HealthReport {
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        engine_ready: true,
        protocols: vec![
            ProtocolEntry {
                id: ProtocolKind::ChatCompletions,
                label: "Chat Completions",
            },
            ProtocolEntry {
                id: ProtocolKind::Responses,
                label: "Responses API",
            },
        ],
    })
}
