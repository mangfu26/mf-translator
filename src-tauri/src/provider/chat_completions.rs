//! OpenAI 兼容协议实现（`POST {base}/chat/completions`，delta 增量 SSE）。
//! 绝大多数第三方供应商与中转服务走此协议。

use futures_util::StreamExt;
use serde_json::{json, Value};
use tokio_util::sync::CancellationToken;

use super::sse::SseDecoder;
use super::{
    transport_error, upstream_error, EventSender, Protocol, ProtocolKind, TranslateRequest,
    TranslationEvent, Usage,
};
use crate::error::{AppError, AppResult};

pub struct ChatCompletions;

pub fn endpoint(base_url: &str) -> String {
    format!("{}/chat/completions", base_url.trim_end_matches('/'))
}

pub fn build_body(request: &TranslateRequest, system: &str, user: &str) -> Value {
    json!({
        "model": request.model,
        "messages": [
            { "role": "system", "content": system },
            { "role": "user", "content": user }
        ],
        "stream": true,
        "stream_options": { "include_usage": true }
    })
}

fn parse_usage(v: &Value) -> Option<Usage> {
    Some(Usage {
        prompt_tokens: v.get("prompt_tokens")?.as_u64()? as u32,
        completion_tokens: v.get("completion_tokens")?.as_u64()? as u32,
    })
}

/// 解析一行 `data:` 载荷，产出 0..n 个翻译事件（一个 chunk 可能同时携带
/// delta 与 usage）。`[DONE]` 与 keepalive 注释行产出空集。
pub fn parse_data_line(line: &str) -> Vec<TranslationEvent> {
    let Some(data) = line.strip_prefix("data:") else {
        return Vec::new();
    };
    let data = data.trim();
    if data.is_empty() || data == "[DONE]" {
        return Vec::new();
    }
    let Ok(v) = serde_json::from_str::<Value>(data) else {
        return Vec::new();
    };

    if let Some(err) = v.get("error") {
        let message = err
            .get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("上游返回错误");
        return vec![TranslationEvent::Failed {
            error: AppError::Network(message.to_string()),
        }];
    }

    let mut events = Vec::new();
    if let Some(text) = v
        .get("choices")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("delta"))
        .and_then(|d| d.get("content"))
        .and_then(|t| t.as_str())
        .filter(|t| !t.is_empty())
    {
        events.push(TranslationEvent::Delta {
            text: text.to_string(),
        });
    }
    if let Some(usage) = v
        .get("usage")
        .filter(|u| !u.is_null())
        .and_then(parse_usage)
    {
        events.push(TranslationEvent::Finished { usage: Some(usage) });
    }
    events
}

#[async_trait::async_trait]
impl Protocol for ChatCompletions {
    fn kind(&self) -> ProtocolKind {
        ProtocolKind::ChatCompletions
    }

    async fn stream(
        &self,
        client: &reqwest::Client,
        base_url: &str,
        api_key: &str,
        request: &TranslateRequest,
        prompts: &super::PromptPair,
        cancel: CancellationToken,
        tx: EventSender,
    ) -> AppResult<()> {
        let system = &prompts.system;
        let user = &prompts.user;

        let mut req = client
            .post(endpoint(base_url))
            .json(&build_body(request, system, user));
        if !api_key.is_empty() {
            req = req.bearer_auth(api_key);
        }

        let response = req.send().await.map_err(transport_error)?;
        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(upstream_error(status, &body));
        }

        let mut stream = response.bytes_stream();
        let mut decoder = SseDecoder::new();
        let mut saw_finish = false;

        loop {
            tokio::select! {
                _ = cancel.cancelled() => return Err(AppError::Cancelled),
                chunk = stream.next() => match chunk {
                    None => break,
                    Some(Ok(bytes)) => {
                        let text = String::from_utf8_lossy(&bytes);
                        for line in decoder.feed(&text) {
                            for event in parse_data_line(&line) {
                                if matches!(event, TranslationEvent::Finished { .. }) {
                                    saw_finish = true;
                                }
                                let _ = tx.send(event);
                            }
                        }
                    }
                    Some(Err(e)) => return Err(transport_error(e)),
                },
            }
        }
        if let Some(line) = decoder.finish() {
            for event in parse_data_line(&line) {
                if matches!(event, TranslationEvent::Finished { .. }) {
                    saw_finish = true;
                }
                let _ = tx.send(event);
            }
        }
        if !saw_finish {
            let _ = tx.send(TranslationEvent::Finished { usage: None });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoint_appends_path_and_trims_slash() {
        assert_eq!(
            endpoint("https://api.example.com/v1/"),
            "https://api.example.com/v1/chat/completions"
        );
    }

    #[test]
    fn parses_delta_chunks() {
        let events = parse_data_line(r#"data: {"choices":[{"delta":{"content":"你"}}]}"#);
        assert_eq!(
            events,
            vec![TranslationEvent::Delta {
                text: "你".to_string()
            }]
        );
    }

    #[test]
    fn parses_usage_chunk_and_done_marker() {
        let events = parse_data_line(
            r#"data: {"choices":[],"usage":{"prompt_tokens":5,"completion_tokens":2}}"#,
        );
        assert_eq!(
            events,
            vec![TranslationEvent::Finished {
                usage: Some(Usage {
                    prompt_tokens: 5,
                    completion_tokens: 2
                })
            }]
        );
        assert!(parse_data_line("data: [DONE]").is_empty());
        assert!(parse_data_line(": keepalive").is_empty());
    }

    #[test]
    fn mid_stream_error_payload_maps_to_failed_event() {
        let events = parse_data_line(r#"data: {"error":{"message":"insufficient quota"}}"#);
        assert!(matches!(
            &events[..],
            [TranslationEvent::Failed { error }] if error.to_string().contains("insufficient quota")
        ));
    }
}
