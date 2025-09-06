use crate::app_error::AppError;
use crate::file_updater::{apply_updates, validate_path};
use crate::response_parser::FileUpdate;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

#[test]
fn test_apply_updates_create_new_file() {
    let dir = tempdir().unwrap();
    let original_cwd = std::env::current_dir().unwrap();
    std::env::set_current_dir(dir.path()).unwrap();

    let file_path = PathBuf::from("new_file.txt");
    let updates = vec![FileUpdate {
        path: file_path.clone(),
        content: "Hello, world!".to_string(),
    }];

    apply_updates(&updates).unwrap();

    assert!(file_path.exists());
    let content = fs::read_to_string(file_path).unwrap();
    assert_eq!(content, "Hello, world!");

    std::env::set_current_dir(original_cwd).unwrap();
}

#[test]
fn test_apply_updates_overwrite_existing_file() {
    let dir = tempdir().unwrap();
    let original_cwd = std::env::current_dir().unwrap();
    std::env::set_current_dir(dir.path()).unwrap();

    let file_path = PathBuf::from("existing.txt");
    fs::write(&file_path, "old content").unwrap();

    let updates = vec![FileUpdate {
        path: file_path.clone(),
        content: "new content".to_string(),
    }];

    apply_updates(&updates).unwrap();

    let content = fs::read_to_string(file_path).unwrap();
    assert_eq!(content, "new content");

    std::env::set_current_dir(original_cwd).unwrap();
}

#[test]
fn test_apply_updates_delete_file() {
    let dir = tempdir().unwrap();
    let original_cwd = std::env::current_dir().unwrap();
    std::env::set_current_dir(dir.path()).unwrap();

    let file_path = PathBuf::from("to_delete.txt");
    fs::write(&file_path, "delete me").unwrap();
    assert!(file_path.exists());

    let updates = vec![FileUpdate {
        path: file_path.clone(),
        content: "".to_string(),
    }];

    apply_updates(&updates).unwrap();

    assert!(!file_path.exists());

    std::env::set_current_dir(original_cwd).unwrap();
}

#[test]
fn test_apply_updates_create_nested_directory() {
    let dir = tempdir().unwrap();
    let original_cwd = std::env::current_dir().unwrap();
    std::env::set_current_dir(dir.path()).unwrap();

    let file_path = PathBuf::from("src").join("app").join("main.rs");

    let updates = vec![FileUpdate {
        path: file_path.clone(),
        content: "fn main() {}".to_string(),
    }];

    apply_updates(&updates).unwrap();
    assert!(file_path.exists());

    std::env::set_current_dir(original_cwd).unwrap();
}

#[test]
fn test_validate_path_valid() {
    assert!(validate_path(&PathBuf::from("src/main.rs")).is_ok());
    assert!(validate_path(&PathBuf::from("a/b/c.txt")).is_ok());
}

#[test]
fn test_validate_path_absolute() {
    let result = validate_path(&PathBuf::from("/etc/passwd"));
    assert!(matches!(result, Err(AppError::FileUpdate(_))));
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Absolute paths are not allowed."));
}

#[test]
fn test_validate_path_traversal() {
    let result = validate_path(&PathBuf::from("../secrets.txt"));
    assert!(matches!(result, Err(AppError::FileUpdate(_))));
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Path traversal ('..') is not allowed."));
}

#[test]
fn test_validate_path_traversal_after_clean() {
    // This path contains traversal components that should be caught.
    let result = validate_path(&PathBuf::from("src/app/../../main.rs"));
    assert!(matches!(result, Err(AppError::FileUpdate(_))));
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Path traversal ('..') is not allowed."));
}

#[test]
fn test_validate_path_git_dir() {
    let result = validate_path(&PathBuf::from(".git/config"));
    assert!(matches!(result, Err(AppError::FileUpdate(_))));
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Modification of '.git' directory is not allowed."));
}
