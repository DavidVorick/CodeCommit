use super::file_updater::{apply_updates, PathProtection};
use super::response_parser::FileUpdate;
use crate::app_error::AppError;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tempfile::{tempdir, TempDir};

// Mutex to ensure that tests changing the current directory don't run in parallel.
static CWD_LOCK: Mutex<()> = Mutex::new(());

struct TestEnv {
    _dir: TempDir,
    original_dir: std::path::PathBuf,
}

impl TestEnv {
    fn new() -> Self {
        let dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();
        TestEnv {
            _dir: dir,
            original_dir,
        }
    }
}

impl Drop for TestEnv {
    fn drop(&mut self) {
        std::env::set_current_dir(&self.original_dir).unwrap();
    }
}

#[test]
fn test_apply_updates_create_new_file() {
    let _lock = CWD_LOCK.lock().unwrap();
    let _env = TestEnv::new();

    let file_path = PathBuf::from("new_file.txt");
    let updates = vec![FileUpdate {
        path: file_path.clone(),
        content: Some("Hello, world!".to_string()),
    }];

    apply_updates(&updates).unwrap();

    assert!(file_path.exists());
    let content = fs::read_to_string(file_path).unwrap();
    assert_eq!(content, "Hello, world!");
}

#[test]
fn test_apply_updates_overwrite_existing_file() {
    let _lock = CWD_LOCK.lock().unwrap();
    let _env = TestEnv::new();

    let file_path = PathBuf::from("existing.txt");
    fs::write(&file_path, "old content").unwrap();

    let updates = vec![FileUpdate {
        path: file_path.clone(),
        content: Some("new content".to_string()),
    }];

    apply_updates(&updates).unwrap();

    let content = fs::read_to_string(file_path).unwrap();
    assert_eq!(content, "new content");
}

#[test]
fn test_apply_updates_create_empty_file() {
    let _lock = CWD_LOCK.lock().unwrap();
    let _env = TestEnv::new();

    let file_path = PathBuf::from("empty_file.txt");
    let updates = vec![FileUpdate {
        path: file_path.clone(),
        content: Some("".to_string()),
    }];

    apply_updates(&updates).unwrap();

    assert!(file_path.exists());
    let content = fs::read_to_string(file_path).unwrap();
    assert_eq!(content, "");
}

#[test]
fn test_apply_updates_delete_file() {
    let _lock = CWD_LOCK.lock().unwrap();
    let _env = TestEnv::new();

    let file_path = PathBuf::from("to_delete.txt");
    fs::write(&file_path, "delete me").unwrap();
    assert!(file_path.exists());

    let updates = vec![FileUpdate {
        path: file_path.clone(),
        content: None,
    }];

    apply_updates(&updates).unwrap();

    assert!(!file_path.exists());
}

#[test]
fn test_apply_updates_create_nested_directory() {
    let _lock = CWD_LOCK.lock().unwrap();
    let _env = TestEnv::new();

    let file_path = PathBuf::from("src").join("app").join("main.rs");

    let updates = vec![FileUpdate {
        path: file_path.clone(),
        content: Some("fn main() {}".to_string()),
    }];

    apply_updates(&updates).unwrap();
    assert!(file_path.exists());
}

#[test]
fn test_validate_path_valid() {
    let protection = PathProtection::new().unwrap();
    assert!(protection.validate(&PathBuf::from("src/main.rs")).is_ok());
    assert!(protection.validate(&PathBuf::from("a/b/c.txt")).is_ok());
}

#[test]
fn test_validate_path_absolute() {
    let protection = PathProtection::new().unwrap();
    let result = protection.validate(&PathBuf::from("/etc/passwd"));
    assert!(matches!(result, Err(AppError::FileUpdate(_))));
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Absolute paths are not allowed."));
}

#[test]
fn test_validate_path_traversal() {
    let protection = PathProtection::new().unwrap();
    let result = protection.validate(&PathBuf::from("../secrets.txt"));
    assert!(matches!(result, Err(AppError::FileUpdate(_))));
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Path traversal ('..') is not allowed."));
}

#[test]
fn test_validate_path_traversal_in_middle() {
    let protection = PathProtection::new().unwrap();
    let result = protection.validate(&PathBuf::from("src/app/../../main.rs"));
    assert!(matches!(result, Err(AppError::FileUpdate(_))));
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Path traversal ('..') is not allowed."));
}

#[test]
fn test_validate_path_forbidden_dir() {
    let protection = PathProtection::new().unwrap();
    let result_git = protection.validate(&PathBuf::from(".git/config"));
    assert!(matches!(result_git, Err(AppError::FileUpdate(_))));
    assert!(result_git
        .unwrap_err()
        .to_string()
        .contains("Modification of directory '.git/' is not allowed."));

    assert!(protection
        .validate(&PathBuf::from("logs/2023-01-01/query.txt"))
        .is_ok());
}

#[test]
fn test_validate_path_forbidden_file() {
    let protection = PathProtection::new().unwrap();
    let result = protection.validate(&PathBuf::from("build.sh"));
    assert!(matches!(result, Err(AppError::FileUpdate(_))));
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Modification of critical file 'build.sh' is not allowed."));

    assert!(protection.validate(&PathBuf::from("query.txt")).is_ok());
}

#[test]
fn test_validate_path_with_gitignore() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join(".gitignore"),
        "# Ignore secrets\nsecrets.txt\n\n# Ignore temp files\n*.tmp\n",
    )
    .unwrap();

    let protection = PathProtection::new_for_base_dir(dir.path()).unwrap();

    let result1 = protection.validate(&PathBuf::from("secrets.txt"));
    assert!(matches!(result1, Err(AppError::FileUpdate(_))));
    assert!(result1.unwrap_err().to_string().contains("secrets.txt"));

    let result2 = protection.validate(&PathBuf::from("data/user.tmp"));
    assert!(matches!(result2, Err(AppError::FileUpdate(_))));
    assert!(result2.unwrap_err().to_string().contains(".tmp"));

    assert!(protection.validate(&PathBuf::from("src/main.rs")).is_ok());
}

#[test]
fn test_forbid_modifying_gitignore_and_agent_config_dir() {
    let protection = PathProtection::new().unwrap();

    let result_gitignore = protection.validate(&PathBuf::from(".gitignore"));
    assert!(matches!(result_gitignore, Err(AppError::FileUpdate(_))));
    assert!(result_gitignore
        .unwrap_err()
        .to_string()
        .contains("Modification of critical file '.gitignore' is not allowed."));

    assert!(protection
        .validate(&PathBuf::from("config/settings.yaml"))
        .is_ok());

    let result_agent_config_dir = protection.validate(&PathBuf::from("agent-config/settings.yaml"));
    assert!(matches!(
        result_agent_config_dir,
        Err(AppError::FileUpdate(_))
    ));
    assert!(result_agent_config_dir
        .unwrap_err()
        .to_string()
        .contains("Modification of directory 'agent-config/' is not allowed."));

    assert!(protection.validate(&PathBuf::from("src/config.rs")).is_ok());
}

#[test]
fn test_validate_user_specification_md_is_forbidden_anywhere() {
    let protection = PathProtection::new().unwrap();

    let root_path = PathBuf::from("UserSpecification.md");
    let result_root = protection.validate(&root_path);
    assert!(matches!(result_root, Err(AppError::FileUpdate(_))));
    assert_eq!(
        result_root.unwrap_err().to_string(),
        "File Update Error: Modification of critical file 'UserSpecification.md' is not allowed."
    );

    let nested_path = PathBuf::from("src/some_module/UserSpecification.md");
    let result_nested = protection.validate(&nested_path);
    assert!(matches!(result_nested, Err(AppError::FileUpdate(_))));
    assert_eq!(
        result_nested.unwrap_err().to_string(),
        "File Update Error: Modification of critical file 'src/some_module/UserSpecification.md' is not allowed."
    );

    let unrelated_path = PathBuf::from("MyUserSpecification.md.bak");
    assert!(protection.validate(&unrelated_path).is_ok());
}

#[test]
fn test_apply_updates_aborts_on_invalid_path_without_applying_changes() {
    let _lock = CWD_LOCK.lock().unwrap();
    let _env = TestEnv::new();

    let valid_path1 = PathBuf::from("valid1.txt");
    let invalid_path = PathBuf::from("build.sh"); // This is a protected file
    let valid_path2 = PathBuf::from("valid2.txt");

    let updates = vec![
        FileUpdate {
            path: valid_path1.clone(),
            content: Some("content1".to_string()),
        },
        FileUpdate {
            path: invalid_path,
            content: Some("hacked".to_string()),
        },
        FileUpdate {
            path: valid_path2.clone(),
            content: Some("content2".to_string()),
        },
    ];

    let result = apply_updates(&updates);
    assert!(matches!(result, Err(AppError::FileUpdate(_))));

    assert!(
        !valid_path1.exists(),
        "First valid file should not be created"
    );
    assert!(
        !valid_path2.exists(),
        "Second valid file should not be created"
    );
}
