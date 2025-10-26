//! A permissive path validator used when **reading files into LLM context**.
//!
//! Rules:
//! - Disallow absolute paths and `..` traversal (safety).
//! - Disallow paths matched by `.gitignore` (privacy/secrets safety).
//! - Allow everything else (e.g., `LLMInstructions.md`, `UserSpecification.md`).
//!
//! This is intentionally more permissive than the file *modification* guard.

use crate::app_error::AppError;
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use std::path::{Component, Path};

/// Validates file paths for **context inclusion** only (not modification).
pub(crate) struct ContextPathFilter {
    gitignore_matcher: Gitignore,
}

impl ContextPathFilter {
    /// Construct a new filter that respects the repository's `.gitignore`.
    ///
    /// # Errors
    /// Returns `AppError::Config` if the `.gitignore` matcher cannot be built.
    pub(crate) fn new() -> Result<Self, AppError> {
        let mut builder = GitignoreBuilder::new(".");
        // Even if the file does not exist, the builder will succeed and match nothing.
        builder.add(".gitignore");
        let gitignore_matcher = builder
            .build()
            .map_err(|e| AppError::Config(format!("Failed to build .gitignore matcher: {e}")))?;

        Ok(Self { gitignore_matcher })
    }

    /// Validate a path selected by the preprocessing LLM for inclusion in the
    /// context payload.
    ///
    /// Safety checks:
    /// - Reject absolute paths.
    /// - Reject `..` traversal anywhere.
    /// - Reject any path that matches `.gitignore` (excluding whitelisted exceptions).
    ///
    /// # Errors
    /// Returns `AppError::FileUpdate` describing the blocked condition.
    pub(crate) fn validate(&self, path: &Path) -> Result<(), AppError> {
        // Basic path safety: no absolute paths or parent traversal.
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

        // Enforce .gitignore: ignore entries are *not* allowed in context.
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
