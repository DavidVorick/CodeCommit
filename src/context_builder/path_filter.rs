use crate::app_error::AppError;
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use path_clean::PathClean;
use std::path::{Component, Path};

pub(crate) struct PathFilter {
    gitignore_matcher: Gitignore,
}

impl PathFilter {
    pub(crate) fn new() -> Result<Self, AppError> {
        Self::new_for_base_dir(Path::new("."))
    }

    pub(crate) fn new_for_base_dir(base_dir: &Path) -> Result<Self, AppError> {
        let mut builder = GitignoreBuilder::new(base_dir);
        builder.add(base_dir.join(".gitignore"));
        let gitignore_matcher = builder
            .build()
            .map_err(|e| AppError::Config(format!("Failed to build .gitignore matcher: {e}")))?;

        Ok(Self { gitignore_matcher })
    }

    pub(crate) fn validate(&self, path: &Path) -> Result<(), AppError> {
        let cleaned = path.clean();

        // Detect traversal/absolute on the original input
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

        // Exception: agent-config/query.txt is allowed in context even if ignored
        let allowed_agent_query = Path::new("agent-config").join("query.txt");
        let is_allowed_agent_query = cleaned == allowed_agent_query;

        // Block protected directories (app-data, agent-config), except the allowed agent-config/query.txt
        if let Some(Component::Normal(first_comp)) = cleaned.components().next() {
            if let Some(name) = first_comp.to_str() {
                if matches!(name, "app-data" | "agent-config") && !is_allowed_agent_query {
                    return Err(AppError::FileUpdate(format!(
                        "File '{}' is in a protected directory and cannot be loaded into context.",
                        cleaned.display()
                    )));
                }
            }
        }

        if !is_allowed_agent_query {
            match self
                .gitignore_matcher
                .matched_path_or_any_parents(&cleaned, false)
            {
                ignore::Match::Ignore(_) => Err(AppError::FileUpdate(format!(
                    "File '{}' matches a rule in .gitignore and cannot be loaded into context.",
                    cleaned.display()
                ))),
                ignore::Match::Whitelist(_) | ignore::Match::None => Ok(()),
            }
        } else {
            Ok(())
        }
    }
}
