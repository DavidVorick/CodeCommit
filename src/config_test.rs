use crate::cli::{CliArgs, Model, Workflow};
use crate::config::Config;
use std::fs;
use tempfile::tempdir;

// Helper to create a test environment
fn setup_test_env(dir: &tempfile::TempDir, gitignore_content: &str, query_content: Option<&str>) {
    fs::write(dir.path().join(".gitignore"), gitignore_content).unwrap();
    let agent_config = dir.path().join("agent-config");
    fs::create_dir_all(&agent_config).unwrap();
    fs::write(agent_config.join("gemini-key.txt"), "gemini-key-123").unwrap();
    fs::write(agent_config.join("openai-key.txt"), "openai-key-456").unwrap();
    if let Some(content) = query_content {
        fs::write(agent_config.join("query.txt"), content).unwrap();
    }
}

#[test]
fn test_load_config_commit_code_workflow() {
    let dir = tempdir().unwrap();
    setup_test_env(&dir, "/agent-config", Some("test query"));
    let args = CliArgs {
        model: Model::Gemini3Pro,
        workflow: Workflow::CommitCode,
        force: false,
        light_roll: false,
    };

    let config = Config::load_from_dir(&args, dir.path()).unwrap();
    assert_eq!(config.api_key, "gemini-key-123");
    assert_eq!(config.query, "test query");
    assert!(!config.system_prompts.contains("refactor"));
}

#[test]
fn test_load_config_gemini2_5_pro_workflow() {
    let dir = tempdir().unwrap();
    setup_test_env(&dir, "/agent-config", Some("test query for gemini 2.5"));
    let args = CliArgs {
        model: Model::Gemini2_5Pro,
        workflow: Workflow::CommitCode,
        force: false,
        light_roll: false,
    };

    let config = Config::load_from_dir(&args, dir.path()).unwrap();
    assert_eq!(config.api_key, "gemini-key-123");
    assert_eq!(config.query, "test query for gemini 2.5");
}

#[test]
fn test_load_config_consistency_check_with_query() {
    let dir = tempdir().unwrap();
    setup_test_env(&dir, "/agent-config", Some("check consistency"));
    let args = CliArgs {
        model: Model::default(),
        workflow: Workflow::ConsistencyCheck,
        force: false,
        light_roll: false,
    };

    let config = Config::load_from_dir(&args, dir.path()).unwrap();
    assert_eq!(config.query, "check consistency");
}

#[test]
fn test_load_config_consistency_check_without_query() {
    let dir = tempdir().unwrap();
    setup_test_env(&dir, "/agent-config", None);
    let args = CliArgs {
        model: Model::default(),
        workflow: Workflow::ConsistencyCheck,
        force: false,
        light_roll: false,
    };

    let config = Config::load_from_dir(&args, dir.path()).unwrap();
    assert_eq!(config.query, "");
}

#[test]
fn test_load_config_missing_gitignore() {
    let dir = tempdir().unwrap();
    let agent_config = dir.path().join("agent-config");
    fs::create_dir_all(&agent_config).unwrap();
    fs::write(agent_config.join("gemini-key.txt"), "gemini-key-123").unwrap();

    let args = CliArgs {
        model: Model::default(),
        workflow: Workflow::default(),
        force: false,
        light_roll: false,
    };

    let result = Config::load_from_dir(&args, dir.path());
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("'.gitignore' file not found"));
}

#[test]
fn test_load_config_gitignore_missing_agent_config() {
    let dir = tempdir().unwrap();
    setup_test_env(&dir, "target/", Some("query"));
    let args = CliArgs {
        model: Model::default(),
        workflow: Workflow::default(),
        force: false,
        light_roll: false,
    };

    let result = Config::load_from_dir(&args, dir.path());
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Your .gitignore file must contain the line '/agent-config'"));
}

#[test]
fn test_load_config_missing_api_key_file() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join(".gitignore"), "/agent-config").unwrap();
    let agent_config = dir.path().join("agent-config");
    fs::create_dir_all(&agent_config).unwrap();
    // No key files written

    let args = CliArgs {
        model: Model::Gemini2_5Pro,
        workflow: Workflow::default(),
        force: false,
        light_roll: false,
    };

    let result = Config::load_from_dir(&args, dir.path());
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Failed to read file 'agent-config/gemini-key.txt'"));
}

#[test]
fn test_load_config_missing_query_file_for_commit_code() {
    let dir = tempdir().unwrap();
    setup_test_env(&dir, "/agent-config", None); // No query file
    let args = CliArgs {
        model: Model::default(),
        workflow: Workflow::CommitCode,
        force: false,
        light_roll: false,
    };

    let result = Config::load_from_dir(&args, dir.path());
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Failed to read file 'agent-config/query.txt'"));
}

#[test]
fn test_gitignore_with_trailing_slash() {
    let dir = tempdir().unwrap();
    setup_test_env(&dir, "agent-config/", Some("test query"));
    let args = CliArgs {
        model: Model::Gemini2_5Pro,
        workflow: Workflow::CommitCode,
        force: false,
        light_roll: false,
    };

    let config_result = Config::load_from_dir(&args, dir.path());
    assert!(config_result.is_ok());
}

#[test]
fn test_config_implements_debug() {
    let cfg = Config {
        model: Model::Gemini3Pro,
        api_key: "k".to_string(),
        query: "q".to_string(),
        system_prompts: "s".to_string(),
    };
    let debug_str = format!("{:?}", cfg);
    assert!(debug_str.contains("api_key"));
}
