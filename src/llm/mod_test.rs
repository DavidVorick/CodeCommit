use super::api::LlmApi;
use super::{create_client, generate_request_id, query_internal};
use crate::app_error::AppError;
use crate::cli::Model;
use crate::logger::Logger;
use serde_json::{json, Value};
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use tempfile::tempdir;

struct MockLlmApiClient {
    response: Result<Value, String>,
    extracted_text: Result<String, String>,
    supports_idempotency: bool,
    last_idempotency_key: Arc<Mutex<Option<String>>>,
}

impl MockLlmApiClient {
    fn new(
        response: Result<Value, String>,
        extracted_text: Result<String, String>,
        supports_idempotency: bool,
    ) -> Self {
        Self {
            response,
            extracted_text,
            supports_idempotency,
            last_idempotency_key: Arc::new(Mutex::new(None)),
        }
    }
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
        idempotency_key: Option<&'a str>,
    ) -> Pin<Box<dyn Future<Output = Result<Value, AppError>> + Send + 'a>> {
        let mut lock = self.last_idempotency_key.lock().unwrap();
        *lock = idempotency_key.map(|s| s.to_string());
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

    let client = MockLlmApiClient::new(
        Ok(json!({"result": "ok"})),
        Ok("llm says hi".to_string()),
        true,
    );

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

    // Verify idempotency key was passed and matches requestId
    let used_key = client.last_idempotency_key.lock().unwrap().clone();
    assert!(used_key.is_some());
    assert_eq!(used_key.unwrap(), query_json["requestId"].as_str().unwrap());

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
async fn test_query_internal_no_idempotency() {
    let dir = tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("agent-config/logs")).unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(dir.path()).unwrap();

    let logger = Logger::new("test-run").unwrap();
    let client = MockLlmApiClient::new(
        Ok(json!({"result": "ok"})),
        Ok("hi".to_string()),
        false, // No idempotency
    );

    let _ = query_internal(&client, "prompt", &logger, "4-test").await;

    let used_key = client.last_idempotency_key.lock().unwrap().clone();
    assert!(used_key.is_none());

    std::env::set_current_dir(original_dir).unwrap();
}

#[tokio::test]
async fn test_query_internal_api_error() {
    let dir = tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("agent-config/logs")).unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(dir.path()).unwrap();

    let logger = Logger::new("test-run").unwrap();
    let client = MockLlmApiClient::new(Err("API failed".to_string()), Ok("...".to_string()), false);

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
    let client = MockLlmApiClient::new(
        Ok(json!({"result": "ok"})),
        Err("Bad format".to_string()),
        false,
    );

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

#[test]
fn test_create_client_config() {
    let client = create_client(Model::Gemini3Pro, "key".into());
    assert_eq!(client.get_model_name(), "gemini-3-pro-preview");
    assert_eq!(
        client.get_url(),
        "https://generativelanguage.googleapis.com/v1beta/interactions"
    );
    assert!(!client.supports_idempotency());

    let client = create_client(Model::Gemini2_5Pro, "key".into());
    assert_eq!(client.get_model_name(), "gemini-2.5-pro");
    assert_eq!(
        client.get_url(),
        "https://generativelanguage.googleapis.com/v1beta/interactions"
    );
    assert!(!client.supports_idempotency());

    let client = create_client(Model::Gpt5, "key".into());
    assert_eq!(client.get_model_name(), "gpt-5.2");
    assert_eq!(
        client.get_url(),
        "https://api.openai.com/v1/chat/completions"
    );
    assert!(client.supports_idempotency());
}
