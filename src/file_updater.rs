use crate::app_error::AppError;
use crate::response_parser::FileUpdate;
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use path_clean::PathClean;
use std::collections::HashSet;
use std::fs;
use std::path::{Component, Path, PathBuf};

pub fn apply_updates(updates: &[FileUpdate]) -> Result<(), AppError> {
    let protection_rules = PathProtection::new()?;

    for update in updates {
        protection_rules.validate(&update.path)?;
        let path = update.path.clean();

        match &update.content {
            Some(content_str) => {
                // This handles both creating new files and replacing existing ones.
                // An empty `content_str` correctly creates an empty file.
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
                // This handles file deletion.
                if path.exists() {
                    fs::remove_file(&path).map_err(|e| {
                        AppError::FileUpdate(format!(
                            "Failed to delete file {}: {}",
                            path.display(),
                            e
                        ))
                    })?;
                }
                // If the file doesn't exist, it's not an error. The goal is achieved.
            }
        }
    }
    Ok(())
}

pub(crate) struct PathProtection {
    forbidden_files: HashSet<PathBuf>,
    gitignore_matcher: Gitignore,
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

        let mut builder = GitignoreBuilder::new(".");
        // Add the local .gitignore file if it exists.
        // The `ignore` crate handles the case where the file doesn't exist gracefully.
        builder.add(".gitignore");
        let gitignore_matcher = builder
            .build()
            .map_err(|e| AppError::Config(format!("Failed to build .gitignore matcher: {e}")))?;

        Ok(Self {
            forbidden_files,
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

        // Check for modification of forbidden root directories like .git/, logs/, target/
        // This structure is refactored to satisfy clippy's `collapsible_match` lint.
        if let Some(Component::Normal(first_comp)) = path.components().next() {
            if let Some(name) = first_comp.to_str() {
                if matches!(name, ".git" | "logs" | "target") {
                    return Err(AppError::FileUpdate(format!(
                        "Modification of directory '{name}/' is not allowed."
                    )));
                }
            }
        }

        // Check against .gitignore rules
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
            // Match::Whitelist means it's explicitly un-ignored, which is fine.
            // Match::None means it's not mentioned, which is also fine.
            ignore::Match::Whitelist(_) | ignore::Match::None => {}
        }

        Ok(())
    }
}
