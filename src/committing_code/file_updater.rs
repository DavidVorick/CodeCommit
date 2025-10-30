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

    // First, validate all paths before making any changes.
    for update in updates {
        protection_rules.validate(&update.path)?;
    }

    // If all validations pass, apply the updates.
    for update in updates {
        let path = update.path.clean();

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
                fs::write(&path, content_str).map_err(|e| {
                    AppError::FileUpdate(format!(
                        "Failed to write to file {}: {}",
                        path.display(),
                        e
                    ))
                })?;
            }
            None => {
                if path.exists() {
                    fs::remove_file(&path).map_err(|e| {
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
        let forbidden_files = [".gitignore", "Cargo.lock", "build.sh", "LLMInstructions.md"]
            .iter()
            .map(PathBuf::from)
            .collect();

        let forbidden_filenames = [OsStr::new("UserSpecification.md")]
            .iter()
            .copied()
            .collect();

        let mut builder = GitignoreBuilder::new(".");
        builder.add(".gitignore");
        let gitignore_matcher = builder
            .build()
            .map_err(|e| AppError::Config(format!("Failed to build .gitignore matcher: {e}")))?;

        Ok(Self {
            forbidden_files,
            forbidden_filenames,
            gitignore_matcher,
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

        if let Some(file_name) = path.file_name() {
            if self.forbidden_filenames.contains(file_name) {
                return Err(AppError::FileUpdate(format!(
                    "Modification of critical file '{}' is not allowed.",
                    path.display()
                )));
            }
        }

        if let Some(Component::Normal(first_comp)) = path.components().next() {
            if let Some(name) = first_comp.to_str() {
                if matches!(name, ".git" | "target" | "agent-config") {
                    return Err(AppError::FileUpdate(format!(
                        "Modification of directory '{name}/' is not allowed."
                    )));
                }
            }
        }

        match self
            .gitignore_matcher
            .matched_path_or_any_parents(path, false)
        {
            ignore::Match::Ignore(_) => {
                return Err(AppError::FileUpdate(format!(
                    "File '{}' matches a rule in .gitignore and cannot be modified.",
                    path.display()
                )));
            }
            ignore::Match::Whitelist(_) | ignore::Match::None => {}
        }

        Ok(())
    }
}
