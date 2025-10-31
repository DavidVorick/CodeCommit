use crate::app_error::AppError;
use crate::cli::{CliArgs, Model, Workflow};
use crate::config::Config;
use std::fs;
use std::io::Write;
use tempfile::TempDir;

struct TestSetup {
    dir: TempDir,
}

impl TestSetup {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let agent_config_path = dir.path().join("agent-config");
        fs::create_dir(&agent_config_path).unwrap();

        let key_path = agent_config_path.join("gemini-key.txt");
        let mut key_file = fs::File::create(&key_path).unwrap();
        write!(key_file, "test-key").unwrap();

        let gitignore_path = dir.path().join(".gitignore");
        let mut gitignore_file = fs::File::create(&gitignore_path).unwrap();
        write!(gitignore_file, "/agent-config").unwrap();

        TestSetup { dir }
    }

    fn create_query_file(&self, content: &str) {
        let query_path = self.dir.path().join("agent-config/query.txt");
        let mut query_file = fs::File::create(&query_path).unwrap();
        write!(query_file, "{}", content).unwrap();
    }

    fn base(&self) -> &std::path::Path {
        self.dir.path()
    }
}

#[test]
fn test_load_config_consistency_with_query() {
    let setup = TestSetup::new();
    setup.create_query_file("test query");

    let args = CliArgs {
        model: Model::Gemini2_5Pro,
        workflow: Workflow::ConsistencyCheck,
        refactor: false,
    };

    let config = Config::load_from_dir(&args, setup.base()).unwrap();
    assert_eq!(config.query, "test query");
}

#[test]
fn test_load_config_consistency_without_query() {
    let setup = TestSetup::new();

    let args = CliArgs {
        model: Model::Gemini2_5Pro,
        workflow: Workflow::ConsistencyCheck,
        refactor: false,
    };

    let config = Config::load_from_dir(&args, setup.base()).unwrap();
    assert_eq!(config.query, "");
}

#[test]
fn test_load_config_commit_code_with_query() {
    let setup = TestSetup::new();
    setup.create_query_file("commit code query");

    let args = CliArgs {
        model: Model::Gemini2_5Pro,
        workflow: Workflow::CommitCode,
        refactor: false,
    };

    let config = Config::load_from_dir(&args, setup.base()).unwrap();
    assert_eq!(config.query, "commit code query");
}

#[test]
fn test_load_config_commit_code_without_query() {
    let setup = TestSetup::new();

    let args = CliArgs {
        model: Model::Gemini2_5Pro,
        workflow: Workflow::CommitCode,
        refactor: false,
    };

    let result = Config::load_from_dir(&args, setup.base());
    assert!(result.is_err());
    if let Err(AppError::Config(msg)) = result {
        assert!(msg.contains("Failed to read file 'agent-config/query.txt'"));
    } else {
        panic!("Expected a Config error");
    }
}

#[test]
fn test_load_config_missing_gitignore() {
    let setup = TestSetup::new();
    fs::remove_file(setup.base().join(".gitignore")).unwrap();

    let args = CliArgs {
        model: Model::Gemini2_5Pro,
        workflow: Workflow::CommitCode,
        refactor: false,
    };

    let result = Config::load_from_dir(&args, setup.base());
    assert!(result.is_err());
    if let Err(AppError::Config(msg)) = result {
        assert!(msg.contains("'.gitignore' file not found"));
    } else {
        panic!("Expected a Config error");
    }
}

#[test]
fn test_load_config_gitignore_missing_agent_config() {
    let setup = TestSetup::new();
    let mut gitignore_file = fs::File::create(setup.base().join(".gitignore")).unwrap();
    write!(gitignore_file, "/target").unwrap();

    let args = CliArgs {
        model: Model::Gemini2_5Pro,
        workflow: Workflow::CommitCode,
        refactor: false,
    };

    let result = Config::load_from_dir(&args, setup.base());
    assert!(result.is_err());
    if let Err(AppError::Config(msg)) = result {
        assert!(msg.contains("Your .gitignore file must contain the line '/agent-config'"));
    } else {
        panic!("Expected a Config error");
    }
}

#[test]
fn test_load_config_missing_api_key_gemini() {
    let setup = TestSetup::new();
    fs::remove_file(setup.base().join("agent-config/gemini-key.txt")).unwrap();
    setup.create_query_file("query");

    let args = CliArgs {
        model: Model::Gemini2_5Pro,
        workflow: Workflow::CommitCode,
        refactor: false,
    };

    let result = Config::load_from_dir(&args, setup.base());
    assert!(result.is_err());
    if let Err(AppError::Config(msg)) = result {
        assert!(msg.contains("Failed to read file 'agent-config/gemini-key.txt'"));
    } else {
        panic!("Expected a Config error");
    }
}

#[test]
fn test_load_config_gpt5_happy_path() {
    let setup = TestSetup::new();
    setup.create_query_file("query");

    let openai_key_path = setup.base().join("agent-config/openai-key.txt");
    let mut openai_key_file = fs::File::create(openai_key_path).unwrap();
    write!(openai_key_file, "gpt-key").unwrap();

    let args = CliArgs {
        model: Model::Gpt5,
        workflow: Workflow::CommitCode,
        refactor: false,
    };

    let config = Config::load_from_dir(&args, setup.base()).unwrap();
    assert_eq!(config.api_key, "gpt-key");
    assert_eq!(config.model, Model::Gpt5);
}

#[test]
fn test_load_config_missing_api_key_gpt5() {
    let setup = TestSetup::new();
    setup.create_query_file("query");

    let args = CliArgs {
        model: Model::Gpt5,
        workflow: Workflow::CommitCode,
        refactor: false,
    };

    let result = Config::load_from_dir(&args, setup.base());
    assert!(result.is_err());
    if let Err(AppError::Config(msg)) = result {
        assert!(msg.contains("Failed to read file 'agent-config/openai-key.txt'"));
    } else {
        panic!("Expected a Config error");
    }
}
