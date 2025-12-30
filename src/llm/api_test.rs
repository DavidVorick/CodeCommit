use super::api::{
    self, extract_text_from_gemini_response, extract_text_from_gpt_response, GeminiClient,
    GptClient, LlmApiClient, QueryError,
};
use crate::app_error::AppError;
use reqwest::StatusCode;
use serde_json::json;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::mpsc;

// --- Mock Server for Integration Tests ---

async fn start_mock_server_with_capture(
    responses: Vec<(u16, String)>,
) -> (String, mpsc::Receiver<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let url = format!("http://127.0.0.1:{port}");
    let (tx, _rx) = mpsc::channel(100);

    tokio::spawn(async move {
        let mut resp_iter = responses.into_iter();
        while let Ok((mut socket, _)) = listener.accept().await {
            let mut buf = [0u8; 8192];
            let n = socket.read(&mut buf).await.unwrap_or(0);
            if n > 0 {
                let request_str = String::from_utf8_lossy(&buf[..n]).to_string();
                let _ = tx.send(request_str).await;
            }

            if let Some((status, body)) = resp_iter.next() {
                let status_line = match status {
                    200 => "200 OK",
                    500 => "500 Internal Server Error",
                    _ => "200 OK",
                };
                let response = format!(
                    "HTTP/1.1 {}\r\nContent-Type: application/json\r\nConnection: close\r\nContent-Length: {}\r\n\r\n{}",
                    status_line,
                    body.len(),
                    body
                );
                let _ = socket.write_all(response.as_bytes()).await;
            }
        }
    });
    (url, _rx)
}

// Convenience wrapper that discards captured requests
async fn start_mock_server(responses: Vec<(u16, String)>) -> String {
    let (url, _) = start_mock_server_with_capture(responses).await;
    url
}

#[tokio::test]
async fn test_gemini_happy_path_immediate() {
    let success_body = json!({
        "status": "completed",
        "outputs": [{ "type": "text", "text": "Immediate success" }]
    })
    .to_string();

    let url = start_mock_server(vec![(200, success_body)]).await;
    let client = GeminiClient::new_test("key".to_string(), "model", url);
    let api_client = LlmApiClient::Gemini(client);

    let res = api_client
        .query_with_retries(&json!({"input": "hi"}), None)
        .await
        .unwrap();

    let text = api_client.extract_text_from_response(&res).unwrap();
    assert_eq!(text, "Immediate success");
}

#[tokio::test]
async fn test_gemini_happy_path_polling() {
    let processing_body = json!({
        "id": "123",
        "status": "processing"
    })
    .to_string();

    let completed_body = json!({
        "id": "123",
        "status": "completed",
        "outputs": [{ "type": "text", "text": "Polled success" }]
    })
    .to_string();

    // 1. POST returns processing
    // 2. GET returns completed
    let url = start_mock_server(vec![(200, processing_body), (200, completed_body)]).await;
    let client = GeminiClient::new_test("key".to_string(), "model", url);
    let api_client = LlmApiClient::Gemini(client);

    let res = api_client
        .query_with_retries(&json!({"input": "hi"}), None)
        .await
        .unwrap();

    let text = api_client.extract_text_from_response(&res).unwrap();
    assert_eq!(text, "Polled success");
}

#[tokio::test]
async fn test_gpt_retry_flow_and_idempotency() {
    let error_body = "{}".to_string();
    let success_body = json!({
        "choices": [{
            "message": { "content": "Retry success" }
        }]
    })
    .to_string();

    // 1. 500 Error
    // 2. 200 OK
    let (url, mut rx) =
        start_mock_server_with_capture(vec![(500, error_body), (200, success_body)]).await;
    let client = GptClient::new_test("key".to_string(), url);
    let api_client = LlmApiClient::Gpt(client);

    let uuid = "uuid-123";
    let res = api_client
        .query_with_retries(&json!({"msg": "hi"}), Some(uuid))
        .await
        .unwrap();

    let text = api_client.extract_text_from_response(&res).unwrap();
    assert_eq!(text, "Retry success");

    // Verify Idempotency Header consistency
    let req1 = rx.recv().await.unwrap();
    let req2 = rx.recv().await.unwrap();

    let expected_header = format!("idempotency-key: {uuid}");
    assert!(
        req1.to_lowercase().contains(&expected_header),
        "First request must have idempotency key. Got:\n{req1}"
    );
    assert!(
        req2.to_lowercase().contains(&expected_header),
        "Retry request must have same idempotency key. Got:\n{req2}"
    );
}

#[tokio::test]
async fn test_gemini_request_structure_and_auth() {
    let success_body = json!({
        "status": "completed",
        "outputs": [{ "type": "text", "text": "ok" }]
    })
    .to_string();

    let (url, mut rx) = start_mock_server_with_capture(vec![(200, success_body)]).await;
    let client = GeminiClient::new_test("my-secret-key".to_string(), "test-model", url);
    let api_client = LlmApiClient::Gemini(client);

    let _ = api_client
        .query_with_retries(&json!({"input": "hi"}), None)
        .await
        .unwrap();

    let req = rx.recv().await.unwrap();
    assert!(
        req.to_lowercase().contains("x-goog-api-key: my-secret-key"),
        "Gemini request must have api key header"
    );
}

#[tokio::test]
async fn test_gpt_auth_header() {
    let success_body = json!({
        "choices": [{ "message": { "content": "ok" } }]
    })
    .to_string();

    let (url, mut rx) = start_mock_server_with_capture(vec![(200, success_body)]).await;
    let client = GptClient::new_test("sk-gpt-key".to_string(), url);
    let api_client = LlmApiClient::Gpt(client);

    let _ = api_client
        .query_with_retries(&json!({"msg": "hi"}), Some("u"))
        .await
        .unwrap();

    let req = rx.recv().await.unwrap();
    assert!(
        req.to_lowercase()
            .contains("authorization: bearer sk-gpt-key"),
        "GPT request must have bearer token. Got:\n{req}"
    );
}

#[test]
fn test_build_request_body_gemini() {
    let client = GeminiClient::new("k".into(), "gemini-model-x");
    let api_client = LlmApiClient::Gemini(client);
    let body = api_client.build_request_body("test prompt");

    assert_eq!(body["model"], "gemini-model-x");
    assert_eq!(body["input"], "test prompt");
    assert!(
        body.get("stream").is_none(),
        "Gemini must not enable streaming"
    );
    assert!(
        body.get("background").is_none(),
        "Gemini must not use background mode"
    );
}

#[test]
fn test_build_request_body_gpt() {
    let client = GptClient::new("k".into());
    let api_client = LlmApiClient::Gpt(client);
    let body = api_client.build_request_body("test prompt");

    assert_eq!(body["model"], "gpt-5.2");
    let msgs = body["messages"].as_array().unwrap();
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0]["role"], "user");
    assert_eq!(msgs[0]["content"], "test prompt");
}

// --- Original Unit Tests ---

#[test]
fn test_extract_gemini_text_happy_path() {
    let response = json!({
        "outputs": [
            {
                "type": "text",
                "text": "This is the LLM response."
            }
        ]
    });
    let result = extract_text_from_gemini_response(&response).unwrap();
    assert_eq!(result, "This is the LLM response.");
}

#[test]
fn test_extract_gemini_text_no_outputs() {
    let response = json!({ "outputs": [] });
    let result = extract_text_from_gemini_response(&response);
    assert!(matches!(result, Err(AppError::ResponseParsing(_))));
}

#[test]
fn test_extract_gemini_text_missing_outputs_key() {
    let response = json!({ "other_key": "value" });
    let result = extract_text_from_gemini_response(&response);
    assert!(matches!(result, Err(AppError::ResponseParsing(_))));
}

#[test]
fn test_extract_gemini_text_missing_text() {
    let response = json!({
        "outputs": [
            { "type": "text", "not_text": "hello" }
        ]
    });
    let result = extract_text_from_gemini_response(&response);
    assert!(matches!(result, Err(AppError::ResponseParsing(_))));
}

#[test]
fn test_extract_gemini_text_multiple_parts() {
    let response = json!({
        "outputs": [
            {
                "type": "text",
                "text": "This is the first part. "
            },
            {
                "type": "image",
                "data": "..."
            },
            {
                "type": "text",
                "text": "This is the second part."
            }
        ]
    });
    let result = extract_text_from_gemini_response(&response).unwrap();
    assert_eq!(result, "This is the first part. This is the second part.");
}

#[test]
fn test_extract_gpt_text_happy_path() {
    let response = json!({
        "choices": [
            {
                "message": {
                    "content": "This is the GPT response."
                }
            }
        ]
    });
    let result = extract_text_from_gpt_response(&response).unwrap();
    assert_eq!(result, "This is the GPT response.");
}

#[test]
fn test_extract_gpt_text_no_choices() {
    let response = json!({"choices": []});
    let result = extract_text_from_gpt_response(&response);
    assert!(matches!(result, Err(AppError::ResponseParsing(_))));
}

#[test]
fn test_extract_gpt_text_missing_content() {
    let response = json!({
        "choices": [
            {
                "message": {
                    "role": "assistant"
                }
            }
        ]
    });
    let result = extract_text_from_gpt_response(&response);
    assert!(matches!(result, Err(AppError::ResponseParsing(_))));
}

#[test]
fn test_extract_gpt_text_missing_message() {
    let response = json!({
        "choices": [
            {
                "finish_reason": "stop"
            }
        ]
    });
    let result = extract_text_from_gpt_response(&response);
    assert!(matches!(result, Err(AppError::ResponseParsing(_))));
}

#[test]
fn test_retryable_transport_for_gpt() {
    let policy = super::api::RetryPolicy::for_model(&fake_gpt_client());
    let timeout_err = QueryError::Transport {
        is_connect: false,
        is_timeout: true,
        message: "timeout".to_string(),
    };
    assert!(policy.is_retryable(&fake_gpt_client(), &timeout_err));

    let connect_err = QueryError::Transport {
        is_connect: true,
        is_timeout: false,
        message: "connect".to_string(),
    };
    assert!(policy.is_retryable(&fake_gpt_client(), &connect_err));
}

#[test]
fn test_retryable_transport_for_gemini() {
    let policy = super::api::RetryPolicy::for_model(&fake_gemini_client());
    let timeout_err = QueryError::Transport {
        is_connect: false,
        is_timeout: true,
        message: "timeout".to_string(),
    };
    assert!(policy.is_retryable(&fake_gemini_client(), &timeout_err));

    let connect_err = QueryError::Transport {
        is_connect: true,
        is_timeout: false,
        message: "connect".to_string(),
    };
    assert!(policy.is_retryable(&fake_gemini_client(), &connect_err));
}

#[test]
fn test_retryable_http_statuses() {
    let policy_gpt = super::api::RetryPolicy::for_model(&fake_gpt_client());
    for code in [408, 429, 500, 502, 503, 504] {
        let e = QueryError::Http {
            status: StatusCode::from_u16(code).unwrap(),
            body: "x".to_string(),
            retry_after: None,
        };
        assert!(
            policy_gpt.is_retryable(&fake_gpt_client(), &e),
            "code {code} should be retryable"
        );
    }
    let e400 = QueryError::Http {
        status: StatusCode::from_u16(400).unwrap(),
        body: "bad".to_string(),
        retry_after: None,
    };
    assert!(!policy_gpt.is_retryable(&fake_gpt_client(), &e400));
}

#[test]
fn test_invalid_json_not_retryable() {
    let policy_g = super::api::RetryPolicy::for_model(&fake_gpt_client());
    let e = QueryError::InvalidJson {
        body: "{}".to_string(),
        parse_error: "bad".to_string(),
    };
    assert!(!policy_g.is_retryable(&fake_gpt_client(), &e));
}

#[test]
fn test_backoff_increases_with_attempts() {
    let policy = super::api::RetryPolicy {
        max_attempts: 5,
        base_delay: Duration::from_millis(100),
        max_delay: Duration::from_secs(2),
    };
    let d1 = policy.backoff_delay(1);
    let d2 = policy.backoff_delay(2);
    let d3 = policy.backoff_delay(3);
    assert!(d2 >= d1);
    assert!(d3 >= d2);
}

#[test]
fn test_censor_api_key_long_key() {
    let text = "The key is a-very-long-api-key-string";
    let key = "a-very-long-api-key-string";
    let censored = api::censor_api_key(text, key);
    assert_eq!(censored, "The key is ...ring");
}

#[test]
fn test_censor_api_key_short_key() {
    let text = "The key is short";
    let key = "short";
    let censored = api::censor_api_key(text, key);
    assert_eq!(censored, "The key is ...");
}

#[test]
fn test_censor_api_key_empty_key() {
    let text = "The key is ";
    let key = "";
    let censored = api::censor_api_key(text, key);
    assert_eq!(censored, "The key is ");
}

#[test]
fn test_censor_api_key_no_match() {
    let text = "The key is not present";
    let key = "secret";
    let censored = api::censor_api_key(text, key);
    assert_eq!(censored, "The key is not present");
}

fn fake_gpt_client() -> LlmApiClient {
    let inner = GptClient::new(String::new());
    LlmApiClient::Gpt(inner)
}

fn fake_gemini_client() -> LlmApiClient {
    let inner = GeminiClient::new(String::new(), "gemini-test-model");
    LlmApiClient::Gemini(inner)
}
