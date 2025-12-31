use super::agent_actions::AgentActions;
use super::run_with_actions;
use crate::app_error::{AppError, BuildFailure};
use crate::cli::Model;
use crate::config::Config;
use crate::logger::Logger;
use std::collections::VecDeque;
use std::fs;
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

    fn get_captured_prompts(&self) -> Vec<String> {
        self.captured_prompts.lock().unwrap().clone()
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
        if responses.is_empty() {
            panic!("Mock query_llm called more times than expected. Log prefix: {_log_prefix}");
        }
        let res = responses.pop_front().unwrap();
        Box::pin(async move { res })
    }

    fn run_build(&self) -> Result<String, BuildFailure> {
        let mut results = self.build_results.lock().unwrap();
        if results.is_empty() {
            panic!("Mock run_build called more times than expected");
        }
        results.pop_front().unwrap()
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
async fn test_extra_code_query_allows_files_only_in_filenames_section() {
    let dir = tempdir().unwrap();
    let logger = Logger::new_with_root(dir.path(), "test").unwrap();
    let config = create_test_config();

    // Create the file that is listed in FILENAMES but not content
    let lib_rs = dir.path().join("src/lib.rs");
    fs::create_dir_all(lib_rs.parent().unwrap()).unwrap();
    fs::write(&lib_rs, "pub fn lib_fn() {}").unwrap();

    // Codebase has it in FILENAMES but not as a file block
    let codebase =
        "--- FILENAMES ---\nsrc/lib.rs\n--- END FILENAMES ---\n\n--- src/main.rs ---\nfn main() {}"
            .to_string();

    // Use '^^^' to avoid confusing the file parser, as per supervisor instructions.
    let caret_block = "^^^";
    let end_block = "^^^end";
    let pct_block = "%%%";

    // 1. Initial response: fails (requests change)
    let response_1 = format!("{caret_block}src/main.rs\nfn main() {{ error }}\n{end_block}");

    // 2. Extra code response: requests src/lib.rs
    let response_extra = format!("{pct_block}files\nsrc/lib.rs\n{pct_block}end");

    // 3. Repair response: success
    let response_2 = format!("{caret_block}src/main.rs\nfn main() {{}}\n{end_block}");

    let actions = MockAgentActions::new(
        vec![Ok(response_1), Ok(response_extra), Ok(response_2)],
        vec![
            Err(BuildFailure {
                output: "error".to_string(),
            }),
            Ok("EXIT CODE: 0".to_string()),
        ],
    );

    let result = run_with_actions(&logger, &config, codebase, &actions, dir.path()).await;

    assert!(result.is_ok());

    let prompts = actions.get_captured_prompts();
    // Prompt 0: Initial
    // Prompt 1: Extra Code
    // Prompt 2: Repair Query
    assert!(prompts.len() >= 3);
    let repair_prompt = &prompts[2];

    // The repair prompt should now contain the content of src/lib.rs because it was requested
    // and was NOT considered already present (since it was only in FILENAMES)
    assert!(repair_prompt.contains("--- src/lib.rs ---"));
    assert!(repair_prompt.contains("pub fn lib_fn() {}"));
}

#[tokio::test]
async fn test_extra_code_query_skips_missing_files() {
    let dir = tempdir().unwrap();
    let logger = Logger::new_with_root(dir.path(), "test").unwrap();
    let config = create_test_config();

    let codebase = "--- src/main.rs ---\nfn main() {}".to_string();

    // Use '^^^' to avoid confusing the file parser, as per supervisor instructions.
    let caret_block = "^^^";
    let end_block = "^^^end";
    let pct_block = "%%%";

    // 1. Initial response: fails
    let response_1 = format!("{caret_block}src/main.rs\nfn main() {{ error }}\n{end_block}");

    // 2. Extra code response: requests a non-existent file
    let response_extra = format!("{pct_block}files\nsrc/does_not_exist.rs\n{pct_block}end");

    // 3. Repair response: success
    let response_2 = format!("{caret_block}src/main.rs\nfn main() {{}}\n{end_block}");

    let actions = MockAgentActions::new(
        vec![Ok(response_1), Ok(response_extra), Ok(response_2)],
        vec![
            Err(BuildFailure {
                output: "error".to_string(),
            }),
            Ok("EXIT CODE: 0".to_string()),
        ],
    );

    // This should not error out, verifying robust handling of missing files
    let result = run_with_actions(&logger, &config, codebase, &actions, dir.path()).await;

    assert!(result.is_ok());

    let prompts = actions.get_captured_prompts();
    let repair_prompt = &prompts[2];

    // Should NOT contain the missing file header
    assert!(!repair_prompt.contains("--- src/does_not_exist.rs ---"));
}
