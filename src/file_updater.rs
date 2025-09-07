use crate::app_error::AppError;
use crate::response_parser::FileUpdate;
use path_clean::PathClean;
use std::collections::HashSet;
use std::fs;
use std::path::{Component, Path, PathBuf};

pub fn apply_updates(updates: &[FileUpdate]) -> Result<(), AppError> {
    let protection_rules = PathProtection::new()?;

    for update in updates {
        protection_rules.validate(&update.path)?;
        let path = update.path.clean();

        if update.content.is_empty() {
            if path.exists() {
                fs::remove_file(&path).map_err(|e| {
                    AppError::FileUpdate(format!("Failed to delete file {}: {}", path.display(), e))
                })?;
            }
        } else {
            if let Some(parent) = path.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent).map_err(|e| {
                        AppError::FileUpdate(format!(
                            "Failed to create parent directory for {}: {}",
                            path.display(),
                            e
                        ))
                    })?;
                }
            }
            fs::write(&path, &update.content).map_err(|e| {
                AppError::FileUpdate(format!("Failed to write to file {}: {}", path.display(), e))
            })?;
        }
    }
    Ok(())
}

pub(crate) struct PathProtection {
    forbidden_files: HashSet<PathBuf>,
    gitignore_patterns: Vec<String>,
}

impl PathProtection {
    pub(crate) fn new() -> Result<Self, AppError> {
        let forbidden_files = [
            "Cargo.lock",
            "build.sh",
            "codeRollup.sh",
            "codeRollup.txt",
            "query.txt",
            "gemini-key.txt",
            "LLMInstructions.md",
            "UserSpecification.md",
        ]
        .iter()
        .map(PathBuf::from)
        .collect();

        let gitignore_patterns = if Path::new(".gitignore").exists() {
            fs::read_to_string(".gitignore")?
                .lines()
                .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
                .map(String::from)
                .collect()
        } else {
            Vec::new()
        };

        Ok(Self {
            forbidden_files,
            gitignore_patterns,
        })
    }

    pub(crate) fn validate(&self, path: &Path) -> Result<(), AppError> {
        for component in path.components() {
            match component {
                Component::RootDir => {
                    return Err(AppError::FileUpdate(
                        "Absolute paths are not allowed.".to_string(),
                    ))
                }
                Component::ParentDir => {
                    return Err(AppError::FileUpdate(
                        "Path traversal ('..') is not allowed.".to_string(),
                    ))
                }
                _ => {}
            }
        }

        if self.forbidden_files.contains(path) {
            return Err(AppError::FileUpdate(format!(
                "Modification of critical file '{}' is not allowed.",
                path.display()
            )));
        }

        let forbidden_dirs = [".git/", "logs/", "target/"];
        for dir in forbidden_dirs {
            if path.starts_with(dir) {
                // Inlined variable to fix clippy warning.
                return Err(AppError::FileUpdate(format!(
                    "Modification of '{dir}' directory is not allowed."
                )));
            }
        }

        let path_str = path.to_string_lossy();
        for pattern in &self.gitignore_patterns {
            // Improved logic to handle simple wildcards like `*.tmp`
            let simple_pattern = pattern.trim_end_matches('/').trim_start_matches('*');
            if path_str.contains(simple_pattern) {
                return Err(AppError::FileUpdate(format!(
                    "File '{}' may match a .gitignore pattern ('{}') and cannot be modified.",
                    path.display(),
                    pattern
                )));
            }
        }

        Ok(())
    }
}
