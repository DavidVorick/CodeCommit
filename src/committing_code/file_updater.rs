use crate::app_error::AppError;
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use path_clean::PathClean;
use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs;
use std::path::{Component, Path, PathBuf};

use super::response_parser::FileUpdate;

pub(crate) fn apply_updates(updates: &[FileUpdate]) -> Result<(), AppError> {
    let protection_rules = PathProtection::new()?;

    // Clean paths first, then validate all of them before applying any changes
    let mut cleaned_updates: Vec<FileUpdate> = Vec::with_capacity(updates.len());
    for update in updates {
        let cleaned_path = update.path.clean();
        // Validate using both original and cleaned paths to ensure adversarial attempts are caught
        protection_rules.validate_paths(&update.path, &cleaned_path)?;
        cleaned_updates.push(FileUpdate {
            path: cleaned_path,
            content: update.content.clone(),
        });
    }

    // Apply updates only after all validations pass
    for update in &cleaned_updates {
        let path = &update.path;

        match &update.content {
            Some(content_str) => {
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
                fs::write(path, content_str).map_err(|e| {
                    AppError::FileUpdate(format!(
                        "Failed to write to file {}: {}",
                        path.display(),
                        e
                    ))
                })?;
            }
            None => {
                if path.exists() {
                    fs::remove_file(path).map_err(|e| {
                        AppError::FileUpdate(format!(
                            "Failed to delete file {}: {}",
                            path.display(),
                            e
                        ))
                    })?;
                }
            }
        }
    }
    Ok(())
}

pub(crate) struct PathProtection {
    forbidden_files: HashSet<PathBuf>,
    forbidden_filenames: HashSet<&'static OsStr>,
    gitignore_matcher: Gitignore,
}

impl PathProtection {
    pub(crate) fn new() -> Result<Self, AppError> {
        Self::new_for_base_dir(Path::new("."))
    }

    pub(crate) fn new_for_base_dir(base_dir: &Path) -> Result<Self, AppError> {
        let forbidden_files = [".gitignore", "Cargo.lock", "build.sh", "LLMInstructions.md"]
            .iter()
            .map(PathBuf::from)
            .collect();

        let forbidden_filenames = [OsStr::new("UserSpecification.md")]
            .iter()
            .copied()
            .collect();

        let mut builder = GitignoreBuilder::new(base_dir);
        builder.add(base_dir.join(".gitignore"));
        let gitignore_matcher = builder
            .build()
            .map_err(|e| AppError::Config(format!("Failed to build .gitignore matcher: {e}")))?;

        Ok(Self {
            forbidden_files,
            forbidden_filenames,
            gitignore_matcher,
        })
    }

    // Primary validation entry point: cleans, then validates.
    pub(crate) fn validate(&self, path: &Path) -> Result<(), AppError> {
        let cleaned = path.clean();
        self.validate_paths(path, &cleaned)
    }

    // Validate using both the original and cleaned paths.
    // The original is used to detect traversal/absolute attempts,
    // while the cleaned path is used for policy checks.
    pub(crate) fn validate_paths(&self, original: &Path, cleaned: &Path) -> Result<(), AppError> {
        // Detect traversal/absolute on the original input
        for component in original.components() {
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

        // Use cleaned path for all subsequent validations
        if self.forbidden_files.contains(cleaned) {
            return Err(AppError::FileUpdate(format!(
                "Modification of critical file '{}' is not allowed.",
                cleaned.display()
            )));
        }

        if let Some(file_name) = cleaned.file_name() {
            if self.forbidden_filenames.contains(file_name) {
                return Err(AppError::FileUpdate(format!(
                    "Modification of critical file '{}' is not allowed.",
                    cleaned.display()
                )));
            }
        }

        if let Some(Component::Normal(first_comp)) = cleaned.components().next() {
            if let Some(name) = first_comp.to_str() {
                // Protect .git, target, agent-config, and app-data directories
                if matches!(name, ".git" | "target" | "agent-config" | "app-data") {
                    return Err(AppError::FileUpdate(format!(
                        "Modification of directory '{name}/' is not allowed."
                    )));
                }
            }
        }

        match self
            .gitignore_matcher
            .matched_path_or_any_parents(cleaned, false)
        {
            ignore::Match::Ignore(_) => {
                return Err(AppError::FileUpdate(format!(
                    "File '{}' matches a rule in .gitignore and cannot be modified.",
                    cleaned.display()
                )));
            }
            ignore::Match::Whitelist(_) | ignore::Match::None => {}
        }

        Ok(())
    }
}
