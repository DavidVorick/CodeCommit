use crate::app_error::AppError;
use crate::cli::{CliArgs, Model, Workflow};
use crate::config::Config;
use std::fs;
use std::io::Write;
use std::sync::Mutex;
use tempfile::TempDir;

// Mutex to ensure that tests changing the current directory don't run in parallel.
static CWD_LOCK: Mutex<()> = Mutex::new(());

struct TestSetup {
    _dir: TempDir, // Keep TempDir alive for the duration of the test
    original_dir: std::path::PathBuf,
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

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        TestSetup {
            _dir: dir,
            original_dir,
        }
    }

    fn create_query_file(&self, content: &str) {
        let query_path = self._dir.path().join("agent-config/query.txt");
        let mut query_file = fs::File::create(&query_path).unwrap();
        write!(query_file, "{}", content).unwrap();
    }
}

impl Drop for TestSetup {
    fn drop(&mut self) {
        std::env::set_current_dir(&self.original_dir).unwrap();
    }
}

#[test]
fn test_load_config_consistency_with_query() {
    let _guard = CWD_LOCK.lock().unwrap();
    let setup = TestSetup::new();
    setup.create_query_file("test query");

    let args = CliArgs {
        model: Model::Gemini2_5Pro,
        workflow: Workflow::ConsistencyCheck,
        refactor: false,
    };

    let config = Config::load(&args).unwrap();
    assert_eq!(config.query, "test query");
}

#[test]
fn test_load_config_consistency_without_query() {
    let _guard = CWD_LOCK.lock().unwrap();
    let _setup = TestSetup::new();

    let args = CliArgs {
        model: Model::Gemini2_5Pro,
        workflow: Workflow::ConsistencyCheck,
        refactor: false,
    };

    let config = Config::load(&args).unwrap();
    assert_eq!(config.query, "");
}

#[test]
fn test_load_config_commit_code_with_query() {
    let _guard = CWD_LOCK.lock().unwrap();
    let setup = TestSetup::new();
    setup.create_query_file("commit code query");

    let args = CliArgs {
        model: Model::Gemini2_5Pro,
        workflow: Workflow::CommitCode,
        refactor: false,
    };

    let config = Config::load(&args).unwrap();
    assert_eq!(config.query, "commit code query");
}

#[test]
fn test_load_config_commit_code_without_query() {
    let _guard = CWD_LOCK.lock().unwrap();
    let _setup = TestSetup::new();

    let args = CliArgs {
        model: Model::Gemini2_5Pro,
        workflow: Workflow::CommitCode,
        refactor: false,
    };

    let result = Config::load(&args);
    assert!(result.is_err());
    if let Err(AppError::Config(msg)) = result {
        assert!(msg.contains("Failed to read file 'agent-config/query.txt'"));
    } else {
        panic!("Expected a Config error");
    }
}
