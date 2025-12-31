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
    let content = fs::read_to_string(main_rs).unwrap();
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
            Ok("".to_string()), // Extra code response (empty)
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
    let content = fs::read_to_string(main_rs).unwrap();
    assert!(content.contains("println!(\"repaired\")"));
}

#[tokio::test]
async fn test_happy_path_success_with_extra_code() {
    let dir = tempdir().unwrap();
    let logger = Logger::new_with_root(dir.path(), "test").unwrap();
    let config = create_test_config();

    // Create the file that will be requested via extra code
    let extra_file_path = dir.path().join("src").join("helper.rs");
    fs::create_dir_all(extra_file_path.parent().unwrap()).unwrap();
    fs::write(&extra_file_path, "pub fn help() {}").unwrap();

    let codebase = "--- src/main.rs ---\nfn main() {}".to_string();

    let caret_block = "^^^";
    let end_block = "^^^end";
    let pct_block = "%%%";

    // 1. Initial response: introduces bug (missing import)
    let response_1 = format!(
        "{caret_block}src/main.rs\nuse helper::help;\nfn main() {{ help(); }}\n{end_block}"
    );

    // 2. Extra code response: requests src/helper.rs
    let response_extra = format!("{pct_block}files\nsrc/helper.rs\n{pct_block}end");

    // 3. Repair response: fixes it (or just confirms it works now that context is there)
    let response_2 = format!("{caret_block}src/main.rs\nmod helper;\nuse helper::help;\nfn main() {{ help(); }}\n{end_block}");

    let actions = MockAgentActions::new(
        vec![Ok(response_1), Ok(response_extra), Ok(response_2)],
        vec![
            Err(BuildFailure {
                output: "error: unresolved import".to_string(),
            }), // Build 1
            Ok("EXIT CODE: 0".to_string()), // Build 2
        ],
    );

    let result = run_with_actions(&logger, &config, codebase, &actions, dir.path()).await;

    assert!(result.is_ok());

    // Check that the repair prompt contained the extra file content
    let prompts = actions.get_captured_prompts();
    // Prompt 0: Initial
    // Prompt 1: Extra Code Query
    // Prompt 2: Repair Query
    assert!(prompts.len() >= 3);
    let repair_prompt = &prompts[2];
    assert!(repair_prompt.contains("src/helper.rs"));
    assert!(repair_prompt.contains("pub fn help() {}"));
}

#[tokio::test]
async fn test_happy_path_success_max_repairs() {
    let dir = tempdir().unwrap();
    let logger = Logger::new_with_root(dir.path(), "test").unwrap();
    let config = create_test_config();
    let codebase = "fn main() {}".to_string();

    let caret_block = "^^^";
    let end_block = "^^^end";

    // Responses:
    // 1. Initial (Fail)
    // 2. Extra (Empty)
    // 3. Repair 1 (Fail)
    // 4. Extra (Empty)
    // 5. Repair 2 (Fail)
    // 6. Extra (Empty)
    // 7. Repair 3 (Success) - This is the 4th attempt total (1 initial + 3 repairs)

    let bad_resp = format!("{caret_block}src/main.rs\ncompile_error!(\"fail\");\n{end_block}");
    let good_resp = format!("{caret_block}src/main.rs\nfn main() {{}}\n{end_block}");
    let empty_extra = "".to_string();

    let actions = MockAgentActions::new(
        vec![
            Ok(bad_resp.clone()),
            Ok(empty_extra.clone()),
            Ok(bad_resp.clone()),
            Ok(empty_extra.clone()),
            Ok(bad_resp.clone()),
            Ok(empty_extra.clone()),
            Ok(good_resp),
        ],
        vec![
            Err(BuildFailure {
                output: "fail".to_string(),
            }), // Build 1
            Err(BuildFailure {
                output: "fail".to_string(),
            }), // Build 2
            Err(BuildFailure {
                output: "fail".to_string(),
            }), // Build 3
            Ok("EXIT CODE: 0".to_string()), // Build 4
        ],
    );

    let result = run_with_actions(&logger, &config, codebase, &actions, dir.path()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_fail_after_max_repairs() {
    let dir = tempdir().unwrap();
    let logger = Logger::new_with_root(dir.path(), "test").unwrap();
    let config = create_test_config();
    let codebase = "fn main() {}".to_string();

    let caret_block = "^^^";
    let end_block = "^^^end";

    // 1 Initial + 3 Repairs = 4 total attempts.
    // All fail.

    let bad_resp = format!("{caret_block}src/main.rs\ncompile_error!(\"fail\");\n{end_block}");
    let empty_extra = "".to_string();

    let actions = MockAgentActions::new(
        vec![
            Ok(bad_resp.clone()),    // Initial
            Ok(empty_extra.clone()), // Extra 1
            Ok(bad_resp.clone()),    // Repair 1
            Ok(empty_extra.clone()), // Extra 2
            Ok(bad_resp.clone()),    // Repair 2
            Ok(empty_extra.clone()), // Extra 3
            Ok(bad_resp.clone()),    // Repair 3
        ],
        vec![
            Err(BuildFailure {
                output: "fail".to_string(),
            }), // Build 1
            Err(BuildFailure {
                output: "fail".to_string(),
            }), // Build 2
            Err(BuildFailure {
                output: "fail".to_string(),
            }), // Build 3
            Err(BuildFailure {
                output: "fail".to_string(),
            }), // Build 4
        ],
    );

    let result = run_with_actions(&logger, &config, codebase, &actions, dir.path()).await;
    assert!(matches!(result, Err(AppError::MaxAttemptsReached)));
}

#[tokio::test]
async fn test_happy_path_file_deletion_and_creation() {
    let dir = tempdir().unwrap();
    let logger = Logger::new_with_root(dir.path(), "test").unwrap();
    let config = create_test_config();

    let delete_path = dir.path().join("src/delete_me.rs");
    fs::create_dir_all(delete_path.parent().unwrap()).unwrap();
    fs::write(&delete_path, "fn old() {}").unwrap();

    let codebase = "--- src/delete_me.rs ---\nfn old() {}".to_string();

    let caret_block = "^^^";
    let end_block = "^^^end";

    // The response deletes 'src/delete_me.rs' and creates 'src/new_file.rs'
    let response = format!(
        "{caret_block}src/delete_me.rs\n{caret_block}delete\n{caret_block}src/new_file.rs\nfn new() {{}}\n{end_block}"
    );

    let actions = MockAgentActions::new(vec![Ok(response)], vec![Ok("EXIT CODE: 0".to_string())]);

    let result = run_with_actions(&logger, &config, codebase, &actions, dir.path()).await;
    assert!(result.is_ok());

    assert!(!delete_path.exists());
    assert!(dir.path().join("src/new_file.rs").exists());
}

#[tokio::test]
async fn test_repair_propt_shows_deletion() {
    let dir = tempdir().unwrap();
    let logger = Logger::new_with_root(dir.path(), "test").unwrap();
    let config = create_test_config();

    let delete_path = dir.path().join("src/delete_me.rs");
    fs::create_dir_all(delete_path.parent().unwrap()).unwrap();
    fs::write(&delete_path, "fn old() {}").unwrap();

    let codebase = "--- src/delete_me.rs ---\nfn old() {}".to_string();

    let caret_block = "^^^";
    let end_block = "^^^end";

    // 1. Initial: delete file
    let response_1 = format!("{caret_block}src/delete_me.rs\n{caret_block}delete");

    // 2. Extra: empty
    let empty_extra = "".to_string();

    // 3. Repair: success (just to end it)
    let response_2 = format!("{caret_block}src/main.rs\nfn main() {{}}\n{end_block}");

    let actions = MockAgentActions::new(
        vec![Ok(response_1), Ok(empty_extra), Ok(response_2)],
        vec![
            Err(BuildFailure {
                output: "error".to_string(),
            }), // Build 1 fails
            Ok("EXIT CODE: 0".to_string()), // Build 2 succeeds
        ],
    );

    let result = run_with_actions(&logger, &config, codebase, &actions, dir.path()).await;
    assert!(result.is_ok());

    let prompts = actions.get_captured_prompts();
    // Prompt 0: Initial
    // Prompt 1: Extra Code
    // Prompt 2: Repair Query (Attempt 2)
    assert!(prompts.len() >= 3);
    let repair_prompt = &prompts[2];

    assert!(repair_prompt.contains("--- FILE REMOVED src/delete_me.rs ---"));
}

#[tokio::test]
async fn test_repair_prompt_accumulates_updates() {
    let dir = tempdir().unwrap();
    let logger = Logger::new_with_root(dir.path(), "test").unwrap();
    let config = create_test_config();
    let codebase = "fn main() {}".to_string();

    let caret_block = "^^^";
    let end_block = "^^^end";

    // 1. Initial: update file1
    let response_1 = format!("{caret_block}src/file1.rs\nfn f1() {{}}\n{end_block}");

    // 2. Extra: empty
    let empty_extra = "".to_string();

    // 3. Repair 1: update file2
    let response_2 = format!("{caret_block}src/file2.rs\nfn f2() {{}}\n{end_block}");

    // 4. Extra: empty

    // 5. Repair 2: success
    let response_3 = format!("{caret_block}src/main.rs\nfn main() {{}}\n{end_block}");

    let actions = MockAgentActions::new(
        vec![
            Ok(response_1),
            Ok(empty_extra.clone()),
            Ok(response_2),
            Ok(empty_extra.clone()),
            Ok(response_3),
        ],
        vec![
            Err(BuildFailure {
                output: "e1".to_string(),
            }), // Build 1
            Err(BuildFailure {
                output: "e2".to_string(),
            }), // Build 2
            Ok("EXIT CODE: 0".to_string()), // Build 3
        ],
    );

    let result = run_with_actions(&logger, &config, codebase, &actions, dir.path()).await;
    assert!(result.is_ok());

    let prompts = actions.get_captured_prompts();
    // Prompt 0: Initial
    // Prompt 1: Extra
    // Prompt 2: Repair 1 (Attempt 2) - Should see file1
    // Prompt 3: Extra
    // Prompt 4: Repair 2 (Attempt 3) - Should see file1 AND file2

    assert!(prompts.len() >= 5);

    let repair_1 = &prompts[2];
    assert!(repair_1.contains("--- FILE REPLACEMENT src/file1.rs ---"));
    assert!(!repair_1.contains("src/file2.rs"));

    let repair_2 = &prompts[4];
    assert!(repair_2.contains("--- FILE REPLACEMENT src/file1.rs ---"));
    assert!(repair_2.contains("--- FILE REPLACEMENT src/file2.rs ---"));
}

#[tokio::test]
async fn test_extra_code_query_ignores_existing_and_protected_files() {
    let dir = tempdir().unwrap();
    let logger = Logger::new_with_root(dir.path(), "test").unwrap();
    let config = create_test_config();

    // Setup files on disk
    let src_dir = dir.path().join("src");
    fs::create_dir_all(&src_dir).unwrap();

    // src/main.rs - already in codebase
    fs::write(src_dir.join("main.rs"), "fn main() {}").unwrap();

    // src/other.rs - valid extra file
    fs::write(src_dir.join("other.rs"), "fn other() {}").unwrap();

    // secret.txt - protected by gitignore
    fs::write(dir.path().join(".gitignore"), "secret.txt").unwrap();
    fs::write(dir.path().join("secret.txt"), "secret").unwrap();

    // Note: run_with_actions takes 'codebase' as string.
    let codebase = "--- src/main.rs ---\nfn main() {}".to_string();

    let caret_block = "^^^";
    let end_block = "^^^end";
    let pct_block = "%%%";

    // 1. Initial response: fails build
    let response_1 = format!("{caret_block}src/main.rs\nfn main() {{ error }}\n{end_block}");

    // 2. Extra code response: requests existing file, protected file, and valid file
    let response_extra =
        format!("{pct_block}files\nsrc/main.rs\nsecret.txt\nsrc/other.rs\n{pct_block}end");

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

    // Check that src/other.rs was added
    assert!(repair_prompt.contains("--- src/other.rs ---"));
    assert!(repair_prompt.contains("fn other() {}"));

    // Check that secret.txt was NOT added
    assert!(!repair_prompt.contains("--- secret.txt ---"));
    assert!(!repair_prompt.contains("secret"));

    // Check that src/main.rs appears exactly twice:
    // 1. In the base codebase section (--- src/main.rs ---)
    // 2. In the file replacements section (--- FILE REPLACEMENT src/main.rs ---)
    let matches: Vec<_> = repair_prompt.match_indices("--- src/main.rs ---").collect();
    assert_eq!(
        matches.len(),
        1,
        "Should contain '--- src/main.rs ---' exactly once (in codebase)"
    );
}

#[tokio::test]
async fn test_repair_prompt_latest_version_only() {
    let dir = tempdir().unwrap();
    let logger = Logger::new_with_root(dir.path(), "test").unwrap();
    let config = create_test_config();
    let codebase = "fn main() {}".to_string();

    let caret_block = "^^^";
    let end_block = "^^^end";

    // Attempt 1: Updates foo.rs with "Version 1"
    let response_1 = format!("{caret_block}src/foo.rs\nfn foo() {{ \"Version 1\" }}\n{end_block}");

    // Extra 1: Empty
    let extra_1 = "".to_string();

    // Attempt 2: Updates foo.rs with "Version 2"
    let response_2 = format!("{caret_block}src/foo.rs\nfn foo() {{ \"Version 2\" }}\n{end_block}");

    // Extra 2: Empty
    let extra_2 = "".to_string();

    // Attempt 3: Success
    let response_3 = format!("{caret_block}src/main.rs\nfn main() {{}}\n{end_block}");

    let actions = MockAgentActions::new(
        vec![
            Ok(response_1),
            Ok(extra_1),
            Ok(response_2),
            Ok(extra_2),
            Ok(response_3),
        ],
        vec![
            Err(BuildFailure {
                output: "fail 1".to_string(),
            }),
            Err(BuildFailure {
                output: "fail 2".to_string(),
            }),
            Ok("EXIT CODE: 0".to_string()),
        ],
    );

    let result = run_with_actions(&logger, &config, codebase, &actions, dir.path()).await;
    assert!(result.is_ok());

    let prompts = actions.get_captured_prompts();
    // Prompt 0: Initial
    // Prompt 1: Extra
    // Prompt 2: Repair 1 (contains Version 1)
    // Prompt 3: Extra
    // Prompt 4: Repair 2 (contains Version 2, should NOT contain Version 1)

    assert!(prompts.len() >= 5);
    let repair_prompt = &prompts[4];

    // It should contain the latest replacement
    assert!(repair_prompt.contains("--- FILE REPLACEMENT src/foo.rs ---"));
    assert!(repair_prompt.contains("Version 2"));

    // It should NOT contain the old replacement
    assert!(!repair_prompt.contains("Version 1"));

    // Verify no duplicate headers
    let matches: Vec<_> = repair_prompt
        .match_indices("--- FILE REPLACEMENT src/foo.rs ---")
        .collect();
    assert_eq!(matches.len(), 1);
}
