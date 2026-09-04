//! OpenAI Responses API 实现（`POST {base}/responses`，类型化 SSE 事件）。
//! 只消费 `response.output_text.delta`，推理模型的思考过程天然被过滤；
//! 请求显式 `store: false`，避免译文在服务商侧留存。

use futures_util::StreamExt;
use serde_json::{json, Value};
use tokio_util::sync::CancellationToken;

use super::sse::SseDecoder;
use super::{
    transport_error, upstream_error, EventSender, Protocol, ProtocolKind, TranslateRequest,
    TranslationEvent, Usage,
};
use crate::error::{AppError, AppResult};

pub struct Responses;

pub fn endpoint(base_url: &str) -> String {
    format!("{}/responses", base_url.trim_end_matches('/'))
}

pub fn build_body(request: &TranslateRequest, instructions: &str, input: &str) -> Value {
    json!({
        "model": request.model,
        "instructions": instructions,
        "input": input,
        "stream": true,
        "store": false
    })
}

fn parse_usage(response: &Value) -> Option<Usage> {
    let usage = response.get("usage")?;
    Some(Usage {
        prompt_tokens: usage.get("input_tokens")?.as_u64()? as u32,
        completion_tokens: usage.get("output_tokens")?.as_u64()? as u32,
    })
}

/// 解析一条 Responses SSE 事件。事件名优先取载荷内 `type` 字段，
/// 缺失时回退到 SSE 的 `event:` 行（部分中转网关只发 data 行）。
pub fn parse_event(event_name: Option<&str>, data: &str) -> Vec<TranslationEvent> {
    let Ok(v) = serde_json::from_str::<Value>(data) else {
        return Vec::new();
    };
    let kind = v
        .get("type")
        .and_then(|t| t.as_str())
        .or(event_name)
        .unwrap_or("");

    match kind {
        "response.output_text.delta" => v
            .get("delta")
            .and_then(|d| d.as_str())
            .filter(|d| !d.is_empty())
            .map(|d| {
                vec![TranslationEvent::Delta {
                    text: d.to_string(),
                }]
            })
            .unwrap_or_default(),
        "response.completed" | "response.incomplete" => v
            .get("response")
            .and_then(parse_usage)
            .map(|usage| vec![TranslationEvent::Finished { usage: Some(usage) }])
            .unwrap_or_else(|| vec![TranslationEvent::Finished { usage: None }]),
        "response.failed" => {
            let message = v
                .pointer("/response/error/message")
                .and_then(|m| m.as_str())
                .unwrap_or("上游处理失败");
            vec![TranslationEvent::Failed {
                error: AppError::Network(message.to_string()),
            }]
        }
        "error" => {
            let message = v
                .pointer("/error/message")
                .or_else(|| v.get("message"))
                .and_then(|m| m.as_str())
                .unwrap_or("上游返回错误");
            vec![TranslationEvent::Failed {
                error: AppError::Network(message.to_string()),
            }]
        }
        _ => Vec::new(),
    }
}

#[async_trait::async_trait]
impl Protocol for Responses {
    fn kind(&self) -> ProtocolKind {
        ProtocolKind::Responses
    }

    async fn stream(
        &self,
        client: &reqwest::Client,
        base_url: &str,
        api_key: &str,
        request: &TranslateRequest,
        cancel: CancellationToken,
        tx: EventSender,
    ) -> AppResult<()> {
        let instructions = crate::translation::prompt::system_prompt(&request.target_language);
        let input = crate::translation::prompt::user_prompt(
            &request.source_text,
            request.source_language.as_deref(),
        );

        let mut req =
            client
                .post(endpoint(base_url))
                .json(&build_body(request, &instructions, &input));
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
        let mut current_event: Option<String> = None;

        let mut handle_line = |line: &str, saw_finish: &mut bool| {
            if let Some(name) = line.strip_prefix("event:") {
                current_event = Some(name.trim().to_string());
                return;
            }
            let Some(data) = line.strip_prefix("data:") else {
                return;
            };
            for event in parse_event(current_event.as_deref(), data.trim()) {
                if matches!(event, TranslationEvent::Finished { .. }) {
                    *saw_finish = true;
                }
                let _ = tx.send(event);
            }
            current_event = None;
        };

        loop {
            tokio::select! {
                _ = cancel.cancelled() => return Err(AppError::Cancelled),
                chunk = stream.next() => match chunk {
                    None => break,
                    Some(Ok(bytes)) => {
                        let text = String::from_utf8_lossy(&bytes);
                        for line in decoder.feed(&text) {
                            handle_line(&line, &mut saw_finish);
                        }
                    }
                    Some(Err(e)) => return Err(transport_error(e)),
                },
            }
        }
        if let Some(line) = decoder.finish() {
            handle_line(&line, &mut saw_finish);
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
            endpoint("https://api.openai.com/v1/"),
            "https://api.openai.com/v1/responses"
        );
    }

    #[test]
    fn parses_delta_via_type_field_without_event_line() {
        let events = parse_event(
            None,
            r#"{"type":"response.output_text.delta","delta":"你好"}"#,
        );
        assert_eq!(
            events,
            vec![TranslationEvent::Delta {
                text: "你好".to_string()
            }]
        );
    }

    #[test]
    fn parses_completed_with_usage() {
        let events = parse_event(
            Some("response.completed"),
            r#"{"type":"response.completed","response":{"usage":{"input_tokens":9,"output_tokens":4}}}"#,
        );
        assert_eq!(
            events,
            vec![TranslationEvent::Finished {
                usage: Some(Usage {
                    prompt_tokens: 9,
                    completion_tokens: 4
                })
            }]
        );
    }

    #[test]
    fn ignores_reasoning_and_other_noise_events() {
        assert!(parse_event(
            None,
            r#"{"type":"response.reasoning_summary_text.delta","delta":"thinking..."}"#
        )
        .is_empty());
        assert!(parse_event(None, r#"{"type":"response.created","response":{}}"#).is_empty());
    }

    #[test]
    fn failed_event_carries_error_message() {
        let events = parse_event(
            Some("response.failed"),
            r#"{"type":"response.failed","response":{"error":{"message":"quota exceeded"}}}"#,
        );
        assert!(matches!(
            &events[..],
            [TranslationEvent::Failed { error }] if error.to_string().contains("quota exceeded")
        ));
    }
}
