use super::context_path_filter::ContextPathFilter;
use crate::app_error::AppError;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

/// Create a temp dir, write a `.gitignore`, build a filter from that directory,
/// then return both so tests can validate behavior deterministically.
fn setup_test_env(gitignore_content: &str) -> (tempfile::TempDir, ContextPathFilter) {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join(".gitignore"), gitignore_content).unwrap();

    let original_cwd = std::env::current_dir().expect("cwd");
    std::env::set_current_dir(dir.path()).unwrap();

    let filter = ContextPathFilter::new().unwrap();

    std::env::set_current_dir(original_cwd).unwrap();
    (dir, filter)
}

#[test]
/// Ensure typical non-ignored project control files (like LLMInstructions.md)
/// are allowed to be included in context.
fn test_allows_llminstructions_not_ignored() {
    let (_dir, filter) = setup_test_env("/agent-config\n/target/\n");
    assert!(filter
        .validate(&PathBuf::from("LLMInstructions.md"))
        .is_ok());
}

#[test]
/// A simple filename in .gitignore must be excluded from context.
fn test_gitignore_simple_filename() {
    let (_dir, filter) = setup_test_env("secrets.txt");
    let res = filter.validate(&PathBuf::from("secrets.txt"));
    assert!(matches!(res, Err(AppError::FileUpdate(_))));
    assert!(filter.validate(&PathBuf::from("src/main.rs")).is_ok());
}

#[test]
/// Wildcard extensions should be respected for exclusion.
fn test_gitignore_wildcard_extension() {
    let (_dir, filter) = setup_test_env("*.log");

    let r1 = filter.validate(&PathBuf::from("debug.log"));
    assert!(matches!(r1, Err(AppError::FileUpdate(_))));

    let r2 = filter.validate(&PathBuf::from("data/error.log"));
    assert!(matches!(r2, Err(AppError::FileUpdate(_))));

    assert!(filter.validate(&PathBuf::from("notes.txt")).is_ok());
}

#[test]
/// Ignored directories (trailing slash) should be excluded recursively.
fn test_gitignore_ignored_directory() {
    let (_dir, filter) = setup_test_env("/target/");

    let r = filter.validate(&PathBuf::from("target/debug/app"));
    assert!(matches!(r, Err(AppError::FileUpdate(_))));

    // Not ignored when 'target' is a non-root nested folder name.
    assert!(filter.validate(&PathBuf::from("src/target/mod.rs")).is_ok());
}

#[test]
/// Negation rules ('!') should whitelist specific files.
fn test_gitignore_negation_rule() {
    let content = "# Ignore logs\n*.log\n# except this one\n!important.log";
    let (_dir, filter) = setup_test_env(content);

    let r1 = filter.validate(&PathBuf::from("debug.log"));
    assert!(matches!(r1, Err(AppError::FileUpdate(_))));

    assert!(filter.validate(&PathBuf::from("important.log")).is_ok());
}

#[test]
/// Root-anchored rules apply only at repository root.
fn test_gitignore_root_specific_rule() {
    let (_dir, filter) = setup_test_env("/config.yaml");

    let r_root = filter.validate(&PathBuf::from("config.yaml"));
    assert!(matches!(r_root, Err(AppError::FileUpdate(_))));

    assert!(filter.validate(&PathBuf::from("src/config.yaml")).is_ok());
}

#[test]
/// Recursive wildcard directory patterns should block all matching subtrees.
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
/// If there's no .gitignore, the filter should permit everything that passes the
/// basic path-safety checks.
fn test_no_gitignore_file_present() {
    let dir = tempdir().unwrap();
    let original_cwd = std::env::current_dir().unwrap();
    std::env::set_current_dir(dir.path()).unwrap();

    let filter = ContextPathFilter::new().unwrap();

    assert!(filter.validate(&PathBuf::from("any/file.txt")).is_ok());
    assert!(filter.validate(&PathBuf::from("another.log")).is_ok());

    std::env::set_current_dir(original_cwd).unwrap();
}

#[test]
/// Basic path-safety checks should still apply for context reads.
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
