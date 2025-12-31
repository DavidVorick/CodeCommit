use super::{ConsistencyDeps, run_internal};
use crate::app_error::AppError;
use crate::cli::Model;
use crate::config::Config;
use crate::logger::Logger;
use crate::system_prompts;
use std::fs;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

struct MockDeps {
    expected_context: String,
    expected_report: String,
    captured_build_context_prompt: Arc<Mutex<Option<String>>>,
    captured_query_llm_prompt: Arc<Mutex<Option<String>>>,
    captured_query_llm_prefix: Arc<Mutex<Option<String>>>,
}

impl MockDeps {
    fn new(context: &str, report: &str) -> Self {
        Self {
            expected_context: context.to_string(),
            expected_report: report.to_string(),
            captured_build_context_prompt: Arc::new(Mutex::new(None)),
            captured_query_llm_prompt: Arc::new(Mutex::new(None)),
            captured_query_llm_prefix: Arc::new(Mutex::new(None)),
        }
    }
}

impl ConsistencyDeps for MockDeps {
    fn build_context<'a>(
        &'a self,
        prompt: &'a str,
        config: &'a Config,
        _logger: &'a Logger,
        prefix: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<String, AppError>> + Send + 'a>> {
        let captured = self.captured_build_context_prompt.clone();
        let prompt_str = prompt.to_string();
        let ret = self.expected_context.clone();
        
        // Assertions for build_context
        assert_eq!(prefix, "1-consistency-context");
        assert!(matches!(config.model, Model::Gemini3Pro));

        Box::pin(async move {
            *captured.lock().unwrap() = Some(prompt_str);
            Ok(ret)
        })
    }

    fn query_llm<'a>(
        &'a self,
        model: Model,
        api_key: String,
        prompt: &'a str,
        _logger: &'a Logger,
        prefix: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<String, AppError>> + Send + 'a>> {
        // Assertions for query_llm arguments
        assert!(matches!(model, Model::Gemini3Pro));
        assert_eq!(api_key, "key");
        
        let captured_prompt = self.captured_query_llm_prompt.clone();
        let captured_prefix = self.captured_query_llm_prefix.clone();
        let prompt_str = prompt.to_string();
        let prefix_str = prefix.to_string();
        let ret = self.expected_report.clone();
        Box::pin(async move {
            *captured_prompt.lock().unwrap() = Some(prompt_str);
            *captured_prefix.lock().unwrap() = Some(prefix_str);
            Ok(ret)
        })
    }
}

#[tokio::test]
async fn test_run_internal_happy_path() {
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let logger = Logger::new_with_root(temp_dir.path(), "test").expect("logger");
    
    let config = Config {
        model: Model::Gemini3Pro,
        api_key: "key".to_string(),
        query: "my query".to_string(),
        system_prompts: "prompts".to_string(),
    };

    let deps = MockDeps::new("mock codebase context", "mock report");
    
    let result = run_internal(&logger, config, &deps).await;
    
    assert!(result.is_ok());

    // Verify build_context inputs
    let build_prompt = deps.captured_build_context_prompt.lock().unwrap().take().expect("build_context should be called");
    let expected_prompt_start = format!(
        "{}\n{}\n[supervisor query]\nmy query",
        system_prompts::PROJECT_STRUCTURE,
        system_prompts::CONSISTENCY_CHECK
    );
    assert_eq!(build_prompt, expected_prompt_start);

    // Verify query_llm inputs
    let query_prompt = deps.captured_query_llm_prompt.lock().unwrap().take().expect("query_llm should be called");
    let expected_query_prompt = format!(
        "{}\n[codebase]\nmock codebase context",
        expected_prompt_start
    );
    assert_eq!(query_prompt, expected_query_prompt);

    let query_prefix = deps.captured_query_llm_prefix.lock().unwrap().take().expect("query_llm should be called");
    assert_eq!(query_prefix, "2-consistency");

    // Verify logging side effect
    // Logger creates a folder [date]-[suffix]
    let mut entries = fs::read_dir(temp_dir.path()).expect("read dir");
    let log_dir_entry = entries.next().expect("should be one entry").expect("entry");
    let log_dir_path = log_dir_entry.path();
    assert!(log_dir_path.is_dir());
    
    let log_file_path = log_dir_path.join("codebase_for_consistency.txt");
    assert!(log_file_path.exists(), "log file should exist");
    let content = fs::read_to_string(log_file_path).expect("read log file");
    assert_eq!(content, "mock codebase context");
}
