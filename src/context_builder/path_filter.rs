use crate::app_error::AppError;
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use std::path::{Component, Path};

pub(crate) struct PathFilter {
    gitignore_matcher: Gitignore,
}

impl PathFilter {
    pub(crate) fn new() -> Result<Self, AppError> {
        let mut builder = GitignoreBuilder::new(".");
        builder.add(".gitignore");
        let gitignore_matcher = builder
            .build()
            .map_err(|e| AppError::Config(format!("Failed to build .gitignore matcher: {e}")))?;

        Ok(Self { gitignore_matcher })
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

        match self
            .gitignore_matcher
            .matched_path_or_any_parents(path, false)
        {
            ignore::Match::Ignore(_) => Err(AppError::FileUpdate(format!(
                "File '{}' matches a rule in .gitignore and cannot be loaded into context.",
                path.display()
            ))),
            ignore::Match::Whitelist(_) | ignore::Match::None => Ok(()),
        }
    }
}
