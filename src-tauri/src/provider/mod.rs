//! 供应商层：对上只暴露统一的 [`Protocol`] trait 与 [`TranslationEvent`] 事件流，
//! 对下由各协议实现封装端点、鉴权与 SSE 解析差异。业务层与前端不感知协议细节。

pub mod chat_completions;
pub mod presets;
pub mod responses;
pub mod sse;

use async_trait::async_trait;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;
use tokio_util::sync::CancellationToken;

use crate::error::{AppError, AppResult};

/// 供应商对话协议。新增协议（如 Anthropic 原生、Gemini 原生）时
/// 增加变体 + 一个 [`Protocol`] 实现 + presets 路由即可，业务层零改动。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ProtocolKind {
    #[default]
    ChatCompletions,
    Responses,
}

/// 一次翻译请求（协议无关的领域模型）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranslateRequest {
    pub model: String,
    pub source_text: String,
    pub target_language: String,
    /// None 表示由模型自动检测源语言。
    pub source_language: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
}

/// 归一化的翻译事件流：前端唯一需要理解的事件协议。
/// 推理类模型的思考过程在协议实现内被过滤，不会进入 Delta。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum TranslationEvent {
    Delta { text: String },
    Finished { usage: Option<Usage> },
    Failed { error: AppError },
}

pub type EventSender = UnboundedSender<TranslationEvent>;

/// 协议实现：把 [`TranslateRequest`] 发往上游，并将响应流归一化为事件序列。
/// `cancel` 被触发时必须尽快中断上游连接并返回 [`AppError::Cancelled`]。
#[async_trait]
pub trait Protocol: Send + Sync {
    fn kind(&self) -> ProtocolKind;

    async fn stream(
        &self,
        client: &reqwest::Client,
        base_url: &str,
        api_key: &str,
        request: &TranslateRequest,
        cancel: CancellationToken,
        tx: EventSender,
    ) -> AppResult<()>;
}

pub use chat_completions::ChatCompletions;
pub use responses::Responses;

pub fn protocol_for(kind: ProtocolKind) -> Box<dyn Protocol> {
    match kind {
        ProtocolKind::ChatCompletions => Box::new(chat_completions::ChatCompletions),
        ProtocolKind::Responses => Box::new(responses::Responses),
    }
}

/// 上游 HTTP 非 2xx 响应 → 归一化 [`AppError`]（解析 OpenAI 风格错误体）。
pub fn upstream_error(status: StatusCode, body: &str) -> AppError {
    let message = serde_json::from_str::<serde_json::Value>(body)
        .ok()
        .and_then(|v| {
            let err = v.get("error");
            err.and_then(|e| e.get("message"))
                .or_else(|| v.get("message"))
                .and_then(|m| m.as_str())
                .map(str::to_string)
        });

    match status {
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => AppError::Auth,
        StatusCode::NOT_FOUND => AppError::Protocol(
            "该服务未提供此协议端点，请在设置中切换协议（Chat Completions / Responses API）"
                .to_string(),
        ),
        StatusCode::TOO_MANY_REQUESTS => {
            AppError::Network("触发供应商限流，请稍后重试".to_string())
        }
        s if s.is_server_error() => {
            AppError::Network(format!("上游服务错误（HTTP {}）", s.as_u16()))
        }
        _ => AppError::Network(
            message.unwrap_or_else(|| format!("请求失败（HTTP {}）", status.as_u16())),
        ),
    }
}

/// 传输层错误（连接失败/超时/中断）→ 归一化 [`AppError`]。
pub fn transport_error(err: reqwest::Error) -> AppError {
    if err.is_connect() {
        AppError::Network("无法连接到服务器，请检查 API 地址与网络".to_string())
    } else if err.is_timeout() {
        AppError::Network("连接超时，请检查网络或稍后重试".to_string())
    } else {
        AppError::Network(format!("网络错误：{err}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unauthorized_maps_to_auth() {
        assert!(matches!(
            upstream_error(
                StatusCode::UNAUTHORIZED,
                r#"{"error":{"message":"bad key"}}"#
            ),
            AppError::Auth
        ));
    }

    #[test]
    fn not_found_maps_to_protocol_hint() {
        let err = upstream_error(StatusCode::NOT_FOUND, "not json");
        assert!(matches!(err, AppError::Protocol(_)));
        assert!(err.to_string().contains("切换协议"));
    }

    #[test]
    fn client_error_extracts_upstream_message() {
        let err = upstream_error(
            StatusCode::BAD_REQUEST,
            r#"{"error":{"message":"model not found","type":"invalid_request_error"}}"#,
        );
        assert!(err.to_string().contains("model not found"));
    }

    #[tokio::test]
    async fn transport_error_classifies_connect_failure() {
        let err = reqwest::get("http://127.0.0.1:1/").await.unwrap_err();
        assert!(matches!(transport_error(err), AppError::Network(m) if m.contains("无法连接")));
    }
}
