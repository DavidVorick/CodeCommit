use super::api::LlmApi;
use super::{generate_request_id, query_internal};
use crate::app_error::AppError;
use crate::logger::Logger;
use serde_json::{json, Value};
use std::future::Future;
use std::pin::Pin;
use tempfile::tempdir;

struct MockLlmApiClient {
    response: Result<Value, String>,
    extracted_text: Result<String, String>,
    supports_idempotency: bool,
}

impl LlmApi for MockLlmApiClient {
    fn get_model_name(&self) -> &'static str {
        "mock-model"
    }
    fn get_url(&self) -> &'static str {
        "http://mock.url"
    }
    fn build_request_body(&self, prompt: &str) -> Value {
        json!({ "prompt": prompt })
    }
    fn query_with_retries<'a>(
        &'a self,
        _request_body: &'a Value,
        _idempotency_key: Option<&'a str>,
    ) -> Pin<Box<dyn Future<Output = Result<Value, AppError>> + Send + 'a>> {
        let resp = self.response.clone().map_err(AppError::Network);
        Box::pin(async { resp })
    }
    fn extract_text_from_response(&self, _response: &Value) -> Result<String, AppError> {
        self.extracted_text
            .clone()
            .map_err(AppError::ResponseParsing)
    }
    fn supports_idempotency(&self) -> bool {
        self.supports_idempotency
    }
}

#[tokio::test]
async fn test_query_internal_happy_path() {
    let dir = tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("agent-config/logs")).unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(dir.path()).unwrap();

    let logger = Logger::new("test-run").unwrap();
    let log_dir_name = std::fs::read_dir("agent-config/logs")
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .file_name()
        .into_string()
        .unwrap();

    let client = MockLlmApiClient {
        response: Ok(json!({"result": "ok"})),
        extracted_text: Ok("llm says hi".to_string()),
        supports_idempotency: true,
    };

    let result = query_internal(&client, "my prompt", &logger, "1-test").await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "llm says hi");

    let log_path = std::path::Path::new("agent-config/logs").join(log_dir_name);
    let query_txt = std::fs::read_to_string(log_path.join("1-test-query.txt")).unwrap();
    assert_eq!(query_txt, "my prompt");

    let query_json: Value =
        serde_json::from_str(&std::fs::read_to_string(log_path.join("1-test-query.json")).unwrap())
            .unwrap();
    assert_eq!(query_json["body"]["prompt"], "my prompt");
    assert!(query_json["requestId"].is_string());

    let response_txt = std::fs::read_to_string(log_path.join("1-test-response.txt")).unwrap();
    assert_eq!(response_txt, "llm says hi");

    let response_json: Value = serde_json::from_str(
        &std::fs::read_to_string(log_path.join("1-test-response.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(response_json["result"], "ok");
    assert!(response_json["totalResponseTime"].is_number());

    std::env::set_current_dir(original_dir).unwrap();
}

#[tokio::test]
async fn test_query_internal_api_error() {
    let dir = tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("agent-config/logs")).unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(dir.path()).unwrap();

    let logger = Logger::new("test-run").unwrap();
    let client = MockLlmApiClient {
        response: Err("API failed".to_string()),
        extracted_text: Ok("...".to_string()),
        supports_idempotency: false,
    };

    let result = query_internal(&client, "prompt", &logger, "2-test").await;
    assert!(result.is_err());
    assert!(matches!(result, Err(AppError::Network(_))));

    std::env::set_current_dir(original_dir).unwrap();
}

#[tokio::test]
async fn test_query_internal_parsing_error() {
    let dir = tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("agent-config/logs")).unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(dir.path()).unwrap();

    let logger = Logger::new("test-run").unwrap();
    let client = MockLlmApiClient {
        response: Ok(json!({"result": "ok"})),
        extracted_text: Err("Bad format".to_string()),
        supports_idempotency: false,
    };

    let result = query_internal(&client, "prompt", &logger, "3-test").await;
    assert!(result.is_err());
    assert!(matches!(result, Err(AppError::ResponseParsing(_))));

    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_generate_request_id() {
    let id1 = generate_request_id();
    let id2 = generate_request_id();
    assert_ne!(id1, id2);
    // UUID v4 is 36 chars long
    assert_eq!(id1.len(), 36);
    assert!(uuid::Uuid::parse_str(&id1).is_ok());
}
