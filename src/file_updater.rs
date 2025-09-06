use crate::app_error::AppError;
use crate::response_parser::FileUpdate;
use path_clean::PathClean;
use std::fs;
use std::path::{Component, Path};

pub fn apply_updates(updates: &[FileUpdate]) -> Result<(), AppError> {
    for update in updates {
        // Security checks on the original, un-normalized path
        validate_path(&update.path)?;

        // Use the cleaned path for all filesystem operations for safety
        let path = update.path.clean();

        if update.content.is_empty() {
            // Delete the file
            if path.exists() {
                fs::remove_file(&path).map_err(|e| {
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
            fs::write(&path, &update.content).map_err(|e| {
                AppError::FileUpdate(format!("Failed to write to file {}: {}", path.display(), e))
            })?;
        }
    }
    Ok(())
}

// Make this function visible within the crate for testing
pub(crate) fn validate_path(path: &Path) -> Result<(), AppError> {
    // Check for forbidden components on the original, uncleaned path.
    // This prevents traversal attacks that might be normalized away by path-clean.
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
