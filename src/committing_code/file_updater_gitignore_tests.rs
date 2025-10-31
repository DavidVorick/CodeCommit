use super::file_updater::PathProtection;
use crate::app_error::AppError;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

fn setup_test_env(gitignore_content: &str) -> (tempfile::TempDir, PathProtection) {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join(".gitignore"), gitignore_content).unwrap();
    let protection = PathProtection::new_for_base_dir(dir.path()).unwrap();
    (dir, protection)
}

#[test]
fn test_gitignore_simple_filename() {
    let (_dir, protection) = setup_test_env("secrets.txt");
    let protected_path = PathBuf::from("secrets.txt");
    let allowed_path = PathBuf::from("src/main.rs");

    let result = protection.validate(&protected_path);
    assert!(matches!(result, Err(AppError::FileUpdate(_))));
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("matches a rule in .gitignore"));

    assert!(protection.validate(&allowed_path).is_ok());
}

#[test]
fn test_gitignore_wildcard_extension() {
    let (_dir, protection) = setup_test_env("*.log");

    let result1 = protection.validate(&PathBuf::from("debug.log"));
    assert!(matches!(result1, Err(AppError::FileUpdate(_))));

    let result2 = protection.validate(&PathBuf::from("data/error.log"));
    assert!(matches!(result2, Err(AppError::FileUpdate(_))));

    assert!(protection.validate(&PathBuf::from("log.txt")).is_ok());
}

#[test]
fn test_gitignore_ignored_directory() {
    let (_dir, protection) = setup_test_env("/target/");

    let result = protection.validate(&PathBuf::from("target/debug/app.exe"));
    assert!(matches!(result, Err(AppError::FileUpdate(_))));

    assert!(protection
        .validate(&PathBuf::from("src/target/mod.rs"))
        .is_ok());
}

#[test]
fn test_gitignore_negation_rule() {
    let gitignore_content = "# Ignore all log files\n*.log\n# Except this one\n!important.log";
    let (_dir, protection) = setup_test_env(gitignore_content);

    let result = protection.validate(&PathBuf::from("debug.log"));
    assert!(matches!(result, Err(AppError::FileUpdate(_))));

    assert!(protection.validate(&PathBuf::from("important.log")).is_ok());
}

#[test]
fn test_gitignore_root_specific_rule() {
    let (_dir, protection) = setup_test_env("/config.yaml");

    let result = protection.validate(&PathBuf::from("config.yaml"));
    assert!(matches!(result, Err(AppError::FileUpdate(_))));

    assert!(protection
        .validate(&PathBuf::from("src/config.yaml"))
        .is_ok());
}

#[test]
fn test_gitignore_comments_and_empty_lines_are_ignored() {
    let gitignore_content = "# This is a comment\n\n  \n*.tmp";
    let (_dir, protection) = setup_test_env(gitignore_content);

    let result = protection.validate(&PathBuf::from("file.tmp"));
    assert!(matches!(result, Err(AppError::FileUpdate(_))));

    assert!(protection.validate(&PathBuf::from("file.txt")).is_ok());
}

#[test]
fn test_gitignore_recursive_wildcard_directory() {
    let (_dir, protection) = setup_test_env("**/temp/");

    let result1 = protection.validate(&PathBuf::from("temp/file.txt"));
    assert!(matches!(result1, Err(AppError::FileUpdate(_))));

    let result2 = protection.validate(&PathBuf::from("src/temp/another.txt"));
    assert!(matches!(result2, Err(AppError::FileUpdate(_))));

    let result3 = protection.validate(&PathBuf::from("src/other/temp/final.txt"));
    assert!(matches!(result3, Err(AppError::FileUpdate(_))));

    assert!(protection
        .validate(&PathBuf::from("src/temporary/file.txt"))
        .is_ok());
}

#[test]
fn test_gitignore_no_gitignore_file_present() {
    let dir = tempdir().unwrap();

    let protection_result = PathProtection::new_for_base_dir(dir.path());
    assert!(protection_result.is_ok());
    let protection = protection_result.unwrap();

    assert!(protection.validate(&PathBuf::from("any/file.txt")).is_ok());
    assert!(protection.validate(&PathBuf::from("another.log")).is_ok());
}
