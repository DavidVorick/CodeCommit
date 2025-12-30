use super::path_filter::PathFilter;
use crate::app_error::AppError;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

fn setup_test_env(gitignore_content: &str) -> (tempfile::TempDir, PathFilter) {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join(".gitignore"), gitignore_content).unwrap();
    let filter = PathFilter::new_for_base_dir(dir.path()).unwrap();
    (dir, filter)
}

#[test]
fn test_allows_llminstructions_not_ignored() {
    let (_dir, filter) = setup_test_env("/agent-config\n/target/\n");
    assert!(filter
        .validate(&PathBuf::from("LLMInstructions.md"))
        .is_ok());
}

#[test]
fn test_gitignore_simple_filename() {
    let (_dir, filter) = setup_test_env("secrets.txt");
    let res = filter.validate(&PathBuf::from("secrets.txt"));
    assert!(matches!(res, Err(AppError::FileUpdate(_))));
    assert!(filter.validate(&PathBuf::from("src/main.rs")).is_ok());
}

#[test]
fn test_gitignore_wildcard_extension() {
    let (_dir, filter) = setup_test_env("*.log");

    let r1 = filter.validate(&PathBuf::from("debug.log"));
    assert!(matches!(r1, Err(AppError::FileUpdate(_))));

    let r2 = filter.validate(&PathBuf::from("data/error.log"));
    assert!(matches!(r2, Err(AppError::FileUpdate(_))));

    assert!(filter.validate(&PathBuf::from("notes.txt")).is_ok());
}

#[test]
fn test_gitignore_ignored_directory() {
    let (_dir, filter) = setup_test_env("/target/");

    let r = filter.validate(&PathBuf::from("target/debug/app"));
    assert!(matches!(r, Err(AppError::FileUpdate(_))));

    assert!(filter.validate(&PathBuf::from("src/target/mod.rs")).is_ok());
}

#[test]
fn test_gitignore_negation_rule() {
    let content = "# Ignore logs\n*.log\n# except this one\n!important.log";
    let (_dir, filter) = setup_test_env(content);

    let r1 = filter.validate(&PathBuf::from("debug.log"));
    assert!(matches!(r1, Err(AppError::FileUpdate(_))));

    assert!(filter.validate(&PathBuf::from("important.log")).is_ok());
}

#[test]
fn test_gitignore_root_specific_rule() {
    let (_dir, filter) = setup_test_env("/config.yaml");

    let r_root = filter.validate(&PathBuf::from("config.yaml"));
    assert!(matches!(r_root, Err(AppError::FileUpdate(_))));

    assert!(filter.validate(&PathBuf::from("src/config.yaml")).is_ok());
}

#[test]
fn test_gitignore_recursive_wildcard_directory() {
    let (_dir, filter) = setup_test_env("**/temp/");

    let r1 = filter.validate(&PathBuf::from("temp/file.txt"));
    assert!(matches!(r1, Err(AppError::FileUpdate(_))));

    let r2 = filter.validate(&PathBuf::from("src/temp/another.txt"));
    assert!(matches!(r2, Err(AppError::FileUpdate(_))));

    let r3 = filter.validate(&PathBuf::from("src/other/temp/final.txt"));
    assert!(matches!(r3, Err(AppError::FileUpdate(_))));

    assert!(filter
        .validate(&PathBuf::from("src/temporary/file.txt"))
        .is_ok());
}

#[test]
fn test_no_gitignore_file_present() {
    let dir = tempdir().unwrap();

    let filter = PathFilter::new_for_base_dir(dir.path()).unwrap();

    assert!(filter.validate(&PathBuf::from("any/file.txt")).is_ok());
    assert!(filter.validate(&PathBuf::from("another.log")).is_ok());
}

#[test]
fn test_absolute_and_traversal_blocked() {
    let (_dir, filter) = setup_test_env("");

    let abs = filter.validate(&PathBuf::from("/etc/passwd"));
    assert!(matches!(abs, Err(AppError::FileUpdate(_))));
    assert!(abs.unwrap_err().to_string().contains("Absolute paths"));

    let traversal = filter.validate(&PathBuf::from("../secrets.txt"));
    assert!(matches!(traversal, Err(AppError::FileUpdate(_))));
    assert!(traversal
        .unwrap_err()
        .to_string()
        .contains("Path traversal ('..')"));
}

#[test]
fn test_agent_config_is_blocked() {
    let (_dir, filter) = setup_test_env("");
    let res = filter.validate(&PathBuf::from("agent-config/query.txt"));
    assert!(matches!(res, Err(AppError::FileUpdate(_))));
    assert!(res.unwrap_err().to_string().contains("protected directory"));
}

#[test]
fn test_app_data_is_blocked_from_context() {
    let (_dir, filter) = setup_test_env("");
    let res = filter.validate(&PathBuf::from("app-data/secret.json"));
    assert!(matches!(res, Err(AppError::FileUpdate(_))));
    assert!(res.unwrap_err().to_string().contains("protected directory"));
}

#[test]
fn test_agent_state_is_blocked_from_context() {
    let (_dir, filter) = setup_test_env("");
    let res = filter.validate(&PathBuf::from("agent-state/session.json"));
    assert!(matches!(res, Err(AppError::FileUpdate(_))));
    assert!(res.unwrap_err().to_string().contains("protected directory"));
}
