use super::summary_builder;
use crate::app_error::AppError;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::sync::Mutex;
use tempfile::TempDir;

// Mutex to ensure that tests changing the current directory don't run in parallel.
static CWD_LOCK: Mutex<()> = Mutex::new(());

struct TestEnv {
    _dir: TempDir,
    original_dir: std::path::PathBuf,
}

impl TestEnv {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();
        TestEnv {
            _dir: dir,
            original_dir,
        }
    }

    fn create_file(&self, path: &str, content: &str) {
        let path = self._dir.path().join(path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        let mut file = fs::File::create(&path).unwrap();
        write!(file, "{content}").unwrap();
    }

    fn path(&self) -> &Path {
        self._dir.path()
    }
}

impl Drop for TestEnv {
    fn drop(&mut self) {
        std::env::set_current_dir(&self.original_dir).unwrap();
    }
}

#[test]
fn test_build_summary_mandatory_file_ignored() {
    let _guard = CWD_LOCK.lock().unwrap();
    let env = TestEnv::new();

    env.create_file(".gitignore", "/target\nCargo.toml");
    env.create_file("Cargo.toml", "[package]");

    let result = summary_builder::build_summary();
    assert!(result.is_err());
    if let Err(AppError::Config(msg)) = result {
        assert!(msg.contains("Mandatory file 'Cargo.toml' is ignored by .gitignore"));
    } else {
        panic!("Expected a Config error, got {result:?}");
    }
}

#[test]
fn test_build_summary_no_src_directory() {
    let _guard = CWD_LOCK.lock().unwrap();
    let env = TestEnv::new();

    env.create_file(".gitignore", "");
    env.create_file("Cargo.toml", "[package]");

    let summary = summary_builder::build_summary().unwrap();

    assert!(summary.contains("--- Cargo.toml ---\n[package]\n\n"));
    assert!(!summary.contains("src/"));
    assert!(!summary.contains("=== src"));
}

#[test]
fn test_build_summary_empty_src_directory() {
    let _guard = CWD_LOCK.lock().unwrap();
    let env = TestEnv::new();

    env.create_file(".gitignore", "");
    env.create_file("Cargo.toml", "[package]");
    fs::create_dir(env.path().join("src")).unwrap();

    let summary = summary_builder::build_summary().unwrap();

    assert!(summary.contains("--- Cargo.toml ---\n[package]\n\n"));
    let filenames_block_start = summary.find("--- FILENAMES ---").unwrap();
    let filenames_block_end = summary.find("--- END FILENAMES ---").unwrap();
    let filenames_block = &summary[filenames_block_start..filenames_block_end];

    assert!(!filenames_block.contains("src"));
    assert!(!summary.contains("=== src"));
}

#[test]
fn test_missing_optional_root_files() {
    let _guard = CWD_LOCK.lock().unwrap();
    let env = TestEnv::new();

    // Only create .gitignore and Cargo.toml
    env.create_file(".gitignore", "");
    env.create_file("Cargo.toml", "[package]");

    let summary = summary_builder::build_summary().unwrap();

    assert!(summary.contains("--- .gitignore ---\n\n\n"));
    assert!(summary.contains("--- Cargo.toml ---\n[package]\n\n"));
    assert!(!summary.contains("--- build.sh ---"));
    assert!(!summary.contains("--- LLMInstructions.md ---"));
    assert!(!summary.contains("--- UserSpecification.md ---"));
}
