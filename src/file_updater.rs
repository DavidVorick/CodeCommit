use crate::app_error::AppError;
use crate::response_parser::FileUpdate;
use path_clean::PathClean;
use std::fs;
use std::path::{Component, Path};

pub fn apply_updates(updates: &[FileUpdate]) -> Result<(), AppError> {
    for update in updates {
        let path = &update.path;

        // Security checks
        validate_path(path)?;

        if update.content.is_empty() {
            // Delete the file
            if path.exists() {
                fs::remove_file(path).map_err(|e| {
                    AppError::FileUpdate(format!("Failed to delete file {}: {}", path.display(), e))
                })?;
            }
        } else {
            // Replace or create the file
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).map_err(|e| {
                    AppError::FileUpdate(format!(
                        "Failed to create parent directory for {}: {}",
                        path.display(),
                        e
                    ))
                })?;
            }
            fs::write(path, &update.content).map_err(|e| {
                AppError::FileUpdate(format!("Failed to write to file {}: {}", path.display(), e))
            })?;
        }
    }
    Ok(())
}

fn validate_path(path: &Path) -> Result<(), AppError> {
    // 1. Normalize path to prevent trivial traversals
    let clean_path = path.clean();

    // 2. Check for forbidden components
    for component in clean_path.components() {
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
            Component::Normal(s) if s.to_str() == Some(".git") => {
                return Err(AppError::FileUpdate(
                    "Modification of '.git' directory is not allowed.".to_string(),
                ))
            }
            _ => {}
        }
    }
    Ok(())
}
