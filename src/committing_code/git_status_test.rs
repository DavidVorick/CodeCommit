#[cfg(test)]
mod tests {
    use super::super::git_status::{check_for_uncommitted_changes, verify_gitignore_protection};
    use crate::app_error::AppError;
    use std::fs;
    use std::path::Path;
    use std::process::Command;
    use std::sync::Mutex;
    use tempfile::{tempdir, TempDir};

    static GIT_TEST_LOCK: Mutex<()> = Mutex::new(());

    struct GitTestEnv {
        _dir: TempDir,
        original_dir: std::path::PathBuf,
    }

    impl GitTestEnv {
        fn new() -> Self {
            let dir = tempdir().unwrap();
            let original_dir = std::env::current_dir().unwrap();
            std::env::set_current_dir(dir.path()).unwrap();
            setup_git_repo(dir.path());
            GitTestEnv {
                _dir: dir,
                original_dir,
            }
        }

        fn path(&self) -> &Path {
            self._dir.path()
        }
    }

    impl Drop for GitTestEnv {
        fn drop(&mut self) {
            std::env::set_current_dir(&self.original_dir).unwrap();
        }
    }

    fn setup_git_repo(path: &Path) {
        Command::new("git")
            .arg("init")
            .current_dir(path)
            .output()
            .expect("Failed to init git repo");
        Command::new("git")
            .arg("config")
            .arg("user.name")
            .arg("Test User")
            .current_dir(path)
            .output()
            .expect("Failed to set git user.name");
        Command::new("git")
            .arg("config")
            .arg("user.email")
            .arg("test@example.com")
            .current_dir(path)
            .output()
            .expect("Failed to set git user.email");
        fs::write(path.join(".gitignore"), "target/\n").unwrap();
    }

    fn commit_all(path: &Path) {
        Command::new("git")
            .arg("add")
            .arg(".")
            .current_dir(path)
            .output()
            .expect("Failed to git add");
        Command::new("git")
            .arg("commit")
            .arg("--allow-empty")
            .arg("-m")
            .arg("initial commit")
            .current_dir(path)
            .output()
            .expect("Failed to git commit");
    }

    #[test]
    fn test_no_changes_is_ok() {
        let _lock = GIT_TEST_LOCK.lock().unwrap();
        let env = GitTestEnv::new();
        fs::write(env.path().join("file.txt"), "hello").unwrap();
        commit_all(env.path());

        let result = check_for_uncommitted_changes();
        assert!(result.is_ok());
    }

    #[test]
    fn test_uncommitted_allowed_file_is_error() {
        let _lock = GIT_TEST_LOCK.lock().unwrap();
        let env = GitTestEnv::new();
        fs::write(env.path().join("file.txt"), "hello").unwrap();
        commit_all(env.path());
        fs::write(env.path().join("file.txt"), "world").unwrap();

        let result = check_for_uncommitted_changes();
        assert!(result.is_err());
        if let Err(AppError::Config(msg)) = result {
            assert!(msg.contains("Uncommitted changes found"));
            assert!(msg.contains("file.txt"));
        } else {
            panic!("Expected Config error");
        }
    }

    #[test]
    fn test_uncommitted_protected_file_is_ok() {
        let _lock = GIT_TEST_LOCK.lock().unwrap();
        let env = GitTestEnv::new();
        fs::write(env.path().join("build.sh"), "#!/bin/bash").unwrap();
        commit_all(env.path());
        fs::write(env.path().join("build.sh"), "#!/bin/bash\necho hello").unwrap();

        let result = check_for_uncommitted_changes();
        assert!(result.is_ok());
    }

    #[test]
    fn test_untracked_allowed_file_is_error() {
        let _lock = GIT_TEST_LOCK.lock().unwrap();
        let env = GitTestEnv::new();
        commit_all(env.path());
        fs::write(env.path().join("new_file.rs"), "fn main() {}").unwrap();

        let result = check_for_uncommitted_changes();
        assert!(result.is_err());
        if let Err(AppError::Config(msg)) = result {
            assert!(msg.contains("new_file.rs"));
        } else {
            panic!("Expected Config error");
        }
    }

    #[test]
    fn test_untracked_protected_file_is_ok() {
        let _lock = GIT_TEST_LOCK.lock().unwrap();
        let env = GitTestEnv::new();
        commit_all(env.path());
        fs::write(env.path().join("UserSpecification.md"), "spec").unwrap();

        let result = check_for_uncommitted_changes();
        assert!(result.is_ok());
    }

    #[test]
    fn test_deleted_allowed_file_is_error() {
        let _lock = GIT_TEST_LOCK.lock().unwrap();
        let env = GitTestEnv::new();
        fs::write(env.path().join("deleteme.txt"), "content").unwrap();
        commit_all(env.path());
        fs::remove_file(env.path().join("deleteme.txt")).unwrap();

        let result = check_for_uncommitted_changes();
        assert!(result.is_err());
        if let Err(AppError::Config(msg)) = result {
            assert!(msg.contains("deleteme.txt"));
        } else {
            panic!("Expected Config error");
        }
    }

    #[test]
    fn test_renamed_allowed_file_is_error() {
        let _lock = GIT_TEST_LOCK.lock().unwrap();
        let env = GitTestEnv::new();
        fs::write(env.path().join("old.txt"), "content").unwrap();
        commit_all(env.path());
        Command::new("git")
            .arg("mv")
            .arg("old.txt")
            .arg("new.txt")
            .current_dir(env.path())
            .output()
            .unwrap();

        let result = check_for_uncommitted_changes();
        assert!(result.is_err());
        if let Err(AppError::Config(msg)) = result {
            assert!(msg.contains("new.txt"));
        } else {
            panic!("Expected Config error");
        }
    }

    #[test]
    fn test_mixed_changes_with_one_allowed_is_error() {
        let _lock = GIT_TEST_LOCK.lock().unwrap();
        let env = GitTestEnv::new();
        fs::write(env.path().join("build.sh"), "").unwrap();
        fs::create_dir_all(env.path().join("src")).unwrap();
        fs::write(env.path().join("src/main.rs"), "").unwrap();
        fs::create_dir_all(env.path().join("agent-config")).unwrap();
        fs::write(env.path().join("agent-config/key.txt"), "").unwrap();
        commit_all(env.path());

        fs::write(env.path().join("build.sh"), "modified").unwrap();
        fs::write(env.path().join("src/main.rs"), "modified").unwrap();
        fs::write(env.path().join("agent-config/key.txt"), "modified").unwrap();

        let result = check_for_uncommitted_changes();
        assert!(result.is_err());
        if let Err(AppError::Config(msg)) = result {
            assert!(msg.contains("src/main.rs"));
            assert!(!msg.contains("build.sh"));
            assert!(!msg.contains("agent-config/key.txt"));
        } else {
            panic!("Expected Config error");
        }
    }

    #[test]
    fn test_gitignore_protection_valid() {
        let _lock = GIT_TEST_LOCK.lock().unwrap();
        let env = GitTestEnv::new();
        // Overwrite default .gitignore with one that protects agent-config
        fs::write(env.path().join(".gitignore"), "target/\n/agent-config").unwrap();

        let result = verify_gitignore_protection();
        assert!(result.is_ok());
    }

    #[test]
    fn test_gitignore_protection_valid_trailing_slash() {
        let _lock = GIT_TEST_LOCK.lock().unwrap();
        let env = GitTestEnv::new();
        fs::write(env.path().join(".gitignore"), "target/\nagent-config/").unwrap();

        let result = verify_gitignore_protection();
        assert!(result.is_ok());
    }

    #[test]
    fn test_gitignore_protection_invalid_missing() {
        let _lock = GIT_TEST_LOCK.lock().unwrap();
        let env = GitTestEnv::new();
        // .gitignore created by setup_git_repo only contains "target/"
        fs::write(env.path().join(".gitignore"), "target/").unwrap();

        let result = verify_gitignore_protection();
        assert!(result.is_err());
        if let Err(AppError::Config(msg)) = result {
            assert!(msg.contains("missing a line for '/agent-config'"));
        } else {
            panic!("Expected Config error");
        }
    }

    #[test]
    fn test_gitignore_protection_file_missing() {
        let _lock = GIT_TEST_LOCK.lock().unwrap();
        let env = GitTestEnv::new();
        fs::remove_file(env.path().join(".gitignore")).unwrap();

        let result = verify_gitignore_protection();
        assert!(result.is_err());
        if let Err(AppError::Config(msg)) = result {
            assert!(msg.contains(".gitignore file is missing"));
        } else {
            panic!("Expected Config error");
        }
    }
}
