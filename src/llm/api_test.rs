use super::api::{
    self, extract_text_from_gemini_response, extract_text_from_gpt_response, LlmApiClient,
    QueryError,
};
use crate::app_error::AppError;
use reqwest::StatusCode;
use serde_json::json;
use std::time::Duration;

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
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Could not find 'content' in GPT response JSON."));
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

// Reliability classification tests
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
    // Gemini interactions are retryable on timeout because we retry the whole flow (safe)
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
    // We only need the enum variant for policy; the inner client is unused here.
    let inner = super::api::GptClient::new(String::new());
    LlmApiClient::Gpt(inner)
}

fn fake_gemini_client() -> LlmApiClient {
    let inner = super::api::GeminiClient::new(String::new(), "gemini-test-model");
    LlmApiClient::Gemini(inner)
}
