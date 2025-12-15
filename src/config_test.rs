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

    // Create .gitignore
    let gitignore_path = base_path.join(".gitignore");
    let mut gitignore_file = File::create(gitignore_path).unwrap();
    writeln!(gitignore_file, "/agent-config").unwrap();

    // Create agent-config dir and key
    let config_dir = base_path.join("agent-config");
    std::fs::create_dir(&config_dir).unwrap();
    let key_path = config_dir.join("gemini-key.txt");
    let mut key_file = File::create(key_path).unwrap();
    write!(key_file, "secret-key").unwrap();

    let args = CliArgs {
        model: Model::Gemini3Pro,
        workflow: Workflow::CommitCode,
        force: false,
        light_roll: false,
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
fn test_load_from_dir_consistency_check_success() {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path();

    // Create .gitignore
    let gitignore_path = base_path.join(".gitignore");
    let mut gitignore_file = File::create(gitignore_path).unwrap();
    writeln!(gitignore_file, "/agent-config").unwrap();

    // Create agent-config dir and key
    let config_dir = base_path.join("agent-config");
    std::fs::create_dir(&config_dir).unwrap();
    let key_path = config_dir.join("gemini-key.txt");
    let mut key_file = File::create(key_path).unwrap();
    write!(key_file, "secret-key").unwrap();

    let args = CliArgs {
        model: Model::Gemini3Pro,
        workflow: Workflow::ConsistencyCheck,
        force: false,
        light_roll: false,
    };

    let query = "consistency query".to_string();
    let config = Config::load_from_dir(&args, base_path, query.clone()).unwrap();

    assert_eq!(config.api_key, "secret-key");
    assert_eq!(config.query, query);
    assert!(!config.system_prompts.is_empty());
}

#[test]
fn test_load_from_dir_missing_gitignore() {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path();

    let args = CliArgs {
        model: Model::Gemini3Pro,
        workflow: Workflow::CommitCode,
        force: false,
        light_roll: false,
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
    writeln!(gitignore_file, "target/").unwrap(); // Missing agent-config

    let args = CliArgs {
        model: Model::Gemini3Pro,
        workflow: Workflow::CommitCode,
        force: false,
        light_roll: false,
    };

    let result = Config::load_from_dir(&args, base_path, "query".to_string());
    assert!(matches!(result, Err(AppError::Config(_))));
}

#[test]
fn test_get_query_from_editor_mocked() {
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
    unsafe {
        std::env::set_var("VISUAL", script_abs_path.to_str().unwrap());
    }

    let query = Config::get_query_from_editor().unwrap();
    assert_eq!(query, "mock query");
}
