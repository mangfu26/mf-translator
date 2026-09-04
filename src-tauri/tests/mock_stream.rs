//! 双协议端到端 mock 测试：用本地 TcpListener 模拟 SSE 上游，
//! 验证 HTTP 调用、鉴权头、SSE 解析与取消语义。

use std::time::Duration;

use mf_translator_lib::error::AppError;
use mf_translator_lib::provider::{
    ChatCompletions, Protocol, Responses, TranslateRequest, TranslationEvent, Usage,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc::UnboundedReceiver;
use tokio_util::sync::CancellationToken;

fn sample_request() -> TranslateRequest {
    TranslateRequest {
        model: "test-model".to_string(),
        source_text: "hello".to_string(),
        target_language: "简体中文".to_string(),
        source_language: None,
    }
}

fn sse_response(body: &str) -> Vec<u8> {
    format!("HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nConnection: close\r\n\r\n{body}")
        .into_bytes()
}

async fn read_headers(sock: &mut tokio::net::TcpStream) -> String {
    let mut buf = Vec::new();
    let mut chunk = [0u8; 4096];
    loop {
        let n = sock.read(&mut chunk).await.expect("读取请求失败");
        buf.extend_from_slice(&chunk[..n]);
        if buf.windows(4).any(|w| w == b"\r\n\r\n") || n == 0 {
            return String::from_utf8_lossy(&buf).to_string();
        }
    }
}

async fn serve_once(
    listener: tokio::net::TcpListener,
    expect_path: &str,
    expect_auth: Option<&str>,
    body: &'static str,
) {
    let (mut sock, _) = listener.accept().await.expect("accept 失败");
    let headers = read_headers(&mut sock).await;
    assert!(headers.contains(expect_path), "请求路径不符：{headers}");
    // reqwest 发送的头部为全小写，断言统一用小写比较
    let headers_lower = headers.to_lowercase();
    match expect_auth {
        Some(auth) => assert!(
            headers_lower.contains(&format!("authorization: bearer {}", auth).to_lowercase()),
            "缺少预期的鉴权头：{headers}"
        ),
        None => assert!(
            !headers_lower.contains("authorization:"),
            "免鉴权请求不应携带 Authorization 头：{headers}"
        ),
    }
    sock.write_all(&sse_response(body)).await.unwrap();
    // Connection: close —— 函数返回即关闭连接，reqwest 读到 EOF 结束
}

async fn drain_events(rx: &mut UnboundedReceiver<TranslationEvent>) -> Vec<TranslationEvent> {
    let mut events = Vec::new();
    let deadline = tokio::time::sleep(Duration::from_secs(10));
    tokio::pin!(deadline);
    loop {
        tokio::select! {
            _ = &mut deadline => panic!("等待事件超时"),
            event = rx.recv() => match event {
                Some(ev) => events.push(ev),
                None => return events,
            },
        }
    }
}

#[tokio::test]
async fn chat_completions_streams_deltas_and_usage() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = tokio::spawn(serve_once(
        listener,
        "/v1/chat/completions",
        Some("sk-test"),
        "data: {\"choices\":[{\"delta\":{\"content\":\"你\"}}]}\n\n\
         data: {\"choices\":[{\"delta\":{\"content\":\"好\"}}]}\n\n\
         data: {\"choices\":[],\"usage\":{\"prompt_tokens\":5,\"completion_tokens\":2}}\n\n\
         data: [DONE]\n\n",
    ));

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let result = ChatCompletions
        .stream(
            &reqwest::Client::new(),
            &format!("http://{addr}/v1"),
            "sk-test",
            &sample_request(),
            CancellationToken::new(),
            tx,
        )
        .await;
    assert!(result.is_ok(), "stream 不应报错");
    server.await.unwrap();

    let events = drain_events(&mut rx).await;
    assert_eq!(
        events,
        vec![
            TranslationEvent::Delta {
                text: "你".to_string()
            },
            TranslationEvent::Delta {
                text: "好".to_string()
            },
            TranslationEvent::Finished {
                usage: Some(Usage {
                    prompt_tokens: 5,
                    completion_tokens: 2
                })
            },
        ]
    );
}

#[tokio::test]
async fn responses_streams_deltas_and_completed() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = tokio::spawn(serve_once(
        listener,
        "/v1/responses",
        Some("sk-test"),
        "data: {\"type\":\"response.created\",\"response\":{}}\n\n\
         data: {\"type\":\"response.output_text.delta\",\"delta\":\"早\"}\n\n\
         data: {\"type\":\"response.output_text.delta\",\"delta\":\"安\"}\n\n\
         data: {\"type\":\"response.completed\",\"response\":{\"usage\":{\"input_tokens\":7,\"output_tokens\":2}}}\n\n",
    ));

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let result = Responses
        .stream(
            &reqwest::Client::new(),
            &format!("http://{addr}/v1"),
            "sk-test",
            &sample_request(),
            CancellationToken::new(),
            tx,
        )
        .await;
    assert!(result.is_ok(), "stream 不应报错");
    server.await.unwrap();

    let events = drain_events(&mut rx).await;
    assert_eq!(
        events,
        vec![
            TranslationEvent::Delta {
                text: "早".to_string()
            },
            TranslationEvent::Delta {
                text: "安".to_string()
            },
            TranslationEvent::Finished {
                usage: Some(Usage {
                    prompt_tokens: 7,
                    completion_tokens: 2
                })
            },
        ]
    );
}

#[tokio::test]
async fn empty_key_omits_authorization_header() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = tokio::spawn(serve_once(
        listener,
        "/v1/chat/completions",
        None,
        "data: {\"choices\":[{\"delta\":{\"content\":\"好\"}}]}\n\n",
    ));

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let result = ChatCompletions
        .stream(
            &reqwest::Client::new(),
            &format!("http://{addr}/v1"),
            "",
            &sample_request(),
            CancellationToken::new(),
            tx,
        )
        .await;
    assert!(result.is_ok());
    server.await.unwrap();

    let events = drain_events(&mut rx).await;
    // 上游未发 usage，实现必须兜底发出 Finished(None)，前端才不会永远等待
    assert_eq!(
        events,
        vec![
            TranslationEvent::Delta {
                text: "好".to_string()
            },
            TranslationEvent::Finished { usage: None },
        ]
    );
}

#[tokio::test]
async fn cancel_interrupts_stalled_stream() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    // 上游只发一个 delta 后挂住连接不关
    tokio::spawn(async move {
        let (mut sock, _) = listener.accept().await.unwrap();
        let _headers = read_headers(&mut sock).await;
        sock.write_all(&sse_response(
            "data: {\"choices\":[{\"delta\":{\"content\":\"你\"}}]}\n\n",
        ))
        .await
        .unwrap();
        tokio::time::sleep(Duration::from_secs(30)).await;
    });

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let cancel = CancellationToken::new();
    let task_cancel = cancel.clone();
    let client = reqwest::Client::new();
    let base_url = format!("http://{addr}/v1");
    let request = sample_request();
    let task = tokio::spawn(async move {
        ChatCompletions
            .stream(&client, &base_url, "sk-test", &request, task_cancel, tx)
            .await
    });

    let first = rx.recv().await.expect("应收到首个 delta");
    assert!(matches!(first, TranslationEvent::Delta { .. }));

    cancel.cancel();
    let result = tokio::time::timeout(Duration::from_secs(5), task)
        .await
        .expect("取消后应尽快返回")
        .expect("任务不应 panic");
    assert!(matches!(result, Err(AppError::Cancelled)));
}
