use super::agent_actions::AgentActions;
use super::run_with_actions;
use crate::app_error::{AppError, BuildFailure};
use crate::cli::Model;
use crate::config::Config;
use crate::logger::Logger;
use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::sync::Mutex;
use tempfile::tempdir;

struct MockAgentActions {
    llm_responses: Mutex<VecDeque<Result<String, AppError>>>,
    build_results: Mutex<VecDeque<Result<String, BuildFailure>>>,
    captured_prompts: Mutex<Vec<String>>,
}

impl MockAgentActions {
    fn new(
        llm_responses: Vec<Result<String, AppError>>,
        build_results: Vec<Result<String, BuildFailure>>,
    ) -> Self {
        Self {
            llm_responses: Mutex::new(llm_responses.into()),
            build_results: Mutex::new(build_results.into()),
            captured_prompts: Mutex::new(Vec::new()),
        }
    }
}

impl AgentActions for MockAgentActions {
    fn query_llm<'a>(
        &'a self,
        _model: Model,
        _api_key: String,
        prompt: String,
        _logger: &'a Logger,
        _log_prefix: String,
    ) -> Pin<Box<dyn Future<Output = Result<String, AppError>> + Send + 'a>> {
        let mut prompts = self.captured_prompts.lock().unwrap();
        prompts.push(prompt);

        let mut responses = self.llm_responses.lock().unwrap();
        let res = responses
            .pop_front()
            .expect("Mock query_llm called more times than expected");
        Box::pin(async move { res })
    }

    fn run_build(&self) -> Result<String, BuildFailure> {
        let mut results = self.build_results.lock().unwrap();
        results
            .pop_front()
            .expect("Mock run_build called more times than expected")
    }
}

fn create_test_config() -> Config {
    Config {
        model: Model::Gpt5,
        api_key: "test-key".to_string(),
        query: "Fix the bug".to_string(),
        system_prompts: "".to_string(),
    }
}

#[tokio::test]
async fn test_happy_path_success_first_try() {
    let dir = tempdir().unwrap();
    let logger = Logger::new_with_root(dir.path(), "test").unwrap();
    let config = create_test_config();
    let codebase = "fn main() {}".to_string();

    let caret_block = "^^^";
    let end_block = "^^^end";
    let llm_response =
        format!("{caret_block}src/main.rs\nfn main() {{ println!(\"fixed\"); }}\n{end_block}");

    let actions = MockAgentActions::new(
        vec![Ok(llm_response)],
        vec![Ok("EXIT CODE: 0\nSTDOUT:\nSTDERR:\n".to_string())],
    );

    let result = run_with_actions(&logger, &config, codebase, &actions, dir.path()).await;

    assert!(result.is_ok());
    let response_text = result.unwrap();
    assert!(response_text.contains("fixed"));

    let main_rs = dir.path().join("src/main.rs");
    let content = std::fs::read_to_string(main_rs).unwrap();
    assert!(content.contains("println!(\"fixed\")"));
}

#[tokio::test]
async fn test_happy_path_success_with_repair() {
    let dir = tempdir().unwrap();
    let logger = Logger::new_with_root(dir.path(), "test").unwrap();
    let config = create_test_config();
    let codebase = "fn main() {}".to_string();

    let caret_block = "^^^";
    let end_block = "^^^end";

    // Attempt 1: Fail
    let response_1 = format!("{caret_block}src/main.rs\nfn main() {{ broken }}\n{end_block}");

    // Attempt 2: Repair success
    let response_2 =
        format!("{caret_block}src/main.rs\nfn main() {{ println!(\"repaired\"); }}\n{end_block}");

    let actions = MockAgentActions::new(
        vec![
            Ok(response_1),
            Ok("".to_string()), // Extra code response
            Ok(response_2),
        ],
        vec![
            Err(BuildFailure {
                output: "EXIT CODE: 1\nerror".to_string(),
            }),
            Ok("EXIT CODE: 0".to_string()),
        ],
    );

    let result = run_with_actions(&logger, &config, codebase, &actions, dir.path()).await;

    assert!(result.is_ok());
    let response_text = result.unwrap();
    assert!(response_text.contains("repaired"));

    let main_rs = dir.path().join("src/main.rs");
    let content = std::fs::read_to_string(main_rs).unwrap();
    assert!(content.contains("println!(\"repaired\")"));
}
