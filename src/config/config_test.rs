use crate::app_error::AppError;
use crate::cli::{CliArgs, Model, Workflow};
use crate::config::Config;
use crate::system_prompts::COMMITTING_CODE_INITIAL_QUERY;
use std::fs::File;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use tempfile::TempDir;

#[test]
fn test_load_from_dir_commit_code_success() {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path();
    setup_valid_environment(base_path, "gemini-key.txt", "secret-key");

    let args = CliArgs {
        model: Model::Gemini3Pro,
        workflow: Workflow::CommitCode,
        force: false,
        rollup_full: false,
    };

    let query = "my query".to_string();
    let config = Config::load_from_dir(&args, base_path, query.clone()).unwrap();

    assert_eq!(config.api_key, "secret-key");
    assert_eq!(config.query, query);
    assert_eq!(
        config.system_prompts.as_str(),
        COMMITTING_CODE_INITIAL_QUERY
    );
}

#[test]
fn test_load_from_dir_gemini_2_5_pro_success() {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path();
    setup_valid_environment(base_path, "gemini-key.txt", "secret-2.5");

    let args = CliArgs {
        model: Model::Gemini2_5Pro,
        workflow: Workflow::CommitCode,
        force: false,
        rollup_full: false,
    };

    let query = "query".to_string();
    let config = Config::load_from_dir(&args, base_path, query.clone()).unwrap();

    assert_eq!(config.api_key, "secret-2.5");
    assert_eq!(config.query, query);
}

#[test]
fn test_load_from_dir_consistency_check_success() {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path();
    setup_valid_environment(base_path, "gemini-key.txt", "secret-key");

    let args = CliArgs {
        model: Model::Gemini3Pro,
        workflow: Workflow::ConsistencyCheck,
        force: false,
        rollup_full: false,
    };

    let query = "consistency query".to_string();
    let config = Config::load_from_dir(&args, base_path, query.clone()).unwrap();

    assert_eq!(config.api_key, "secret-key");
    assert!(!config.system_prompts.is_empty());
}

#[test]
fn test_load_from_dir_auto_workflow_success() {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path();
    setup_valid_environment(base_path, "gemini-key.txt", "gemini-secret-auto");

    let args = CliArgs {
        model: Model::Gemini3Pro,
        workflow: Workflow::Auto,
        force: false,
        rollup_full: false,
    };

    let query = "".to_string();
    let config = Config::load_from_dir(&args, base_path, query.clone()).unwrap();

    assert_eq!(config.api_key, "gemini-secret-auto");
    assert_eq!(config.query, "");
    assert!(config.system_prompts.is_empty());
}

#[test]
fn test_load_from_dir_gpt5_success() {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path();
    setup_valid_environment(base_path, "openai-key.txt", "openai-secret");

    let args = CliArgs {
        model: Model::Gpt5,
        workflow: Workflow::CommitCode,
        force: false,
        rollup_full: false,
    };

    let query = "gpt query".to_string();
    let config = Config::load_from_dir(&args, base_path, query.clone()).unwrap();

    assert_eq!(config.api_key, "openai-secret");
}

#[test]
fn test_load_from_dir_rollup_returns_error() {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path();
    setup_valid_environment(base_path, "gemini-key.txt", "key");

    let args = CliArgs {
        model: Model::Gemini3Pro,
        workflow: Workflow::Rollup,
        force: false,
        rollup_full: false,
    };

    let result = Config::load_from_dir(&args, base_path, "".to_string());
    assert!(matches!(result, Err(AppError::Config(_))));
}

#[test]
fn test_load_from_dir_init_returns_error() {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path();
    setup_valid_environment(base_path, "gemini-key.txt", "key");

    let args = CliArgs {
        model: Model::Gemini3Pro,
        workflow: Workflow::Init("proj".to_string()),
        force: false,
        rollup_full: false,
    };

    let result = Config::load_from_dir(&args, base_path, "".to_string());
    assert!(matches!(result, Err(AppError::Config(_))));
}

#[test]
fn test_load_from_dir_missing_gitignore() {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path();

    let args = CliArgs {
        model: Model::Gemini3Pro,
        workflow: Workflow::CommitCode,
        force: false,
        rollup_full: false,
    };

    let result = Config::load_from_dir(&args, base_path, "query".to_string());
    assert!(matches!(result, Err(AppError::Config(_))));
}

#[test]
fn test_load_from_dir_insecure_gitignore() {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path();

    let gitignore_path = base_path.join(".gitignore");
    let mut gitignore_file = File::create(gitignore_path).unwrap();
    writeln!(gitignore_file, "target/").unwrap();

    let args = CliArgs {
        model: Model::Gemini3Pro,
        workflow: Workflow::CommitCode,
        force: false,
        rollup_full: false,
    };

    let result = Config::load_from_dir(&args, base_path, "query".to_string());
    assert!(matches!(result, Err(AppError::Config(_))));
}

#[test]
fn test_get_query_from_editor_env_handling() {
    let temp_dir = TempDir::new().unwrap();
    let script_path = temp_dir.path().join("mock_editor.sh");

    {
        let mut f = File::create(&script_path).unwrap();
        writeln!(f, "#!/bin/sh").unwrap();
        writeln!(f, "echo 'mock query' > $1").unwrap();
    }

    let mut perms = std::fs::metadata(&script_path).unwrap().permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&script_path, perms).unwrap();

    let script_abs_path = script_path.canonicalize().unwrap();
    let script_str = script_abs_path.to_str().unwrap();

    // Case 1: VISUAL is set.
    unsafe {
        std::env::set_var("VISUAL", script_str);
    }
    // We set EDITOR to something invalid to prove it is ignored when VISUAL is present.
    unsafe {
        std::env::set_var("EDITOR", "/invalid/path");
    }

    let query = Config::get_query_from_editor().unwrap();
    assert_eq!(query, "mock query");

    // Case 2: VISUAL is unset, EDITOR is set.
    unsafe {
        std::env::remove_var("VISUAL");
    }
    unsafe {
        std::env::set_var("EDITOR", script_str);
    }

    let query = Config::get_query_from_editor().unwrap();
    assert_eq!(query, "mock query");

    // Cleanup
    unsafe {
        std::env::remove_var("VISUAL");
    }
    unsafe {
        std::env::remove_var("EDITOR");
    }
}

fn setup_valid_environment(base_path: &std::path::Path, key_file: &str, key_content: &str) {
    let gitignore_path = base_path.join(".gitignore");
    let mut gitignore_file = File::create(gitignore_path).unwrap();
    writeln!(gitignore_file, "/agent-config").unwrap();

    let config_dir = base_path.join("agent-config");
    std::fs::create_dir(&config_dir).unwrap();
    let key_path = config_dir.join(key_file);
    let mut key_file = File::create(key_path).unwrap();
    write!(key_file, "{key_content}").unwrap();
}
