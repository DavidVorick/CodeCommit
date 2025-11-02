use crate::app_error::AppError;
use crate::committing_code::file_updater::PathProtection;
use std::collections::HashSet;
use std::path::PathBuf;
use std::process::Command;

pub(crate) fn check_for_uncommitted_changes() -> Result<(), AppError> {
    let output = Command::new("git")
        .arg("status")
        .arg("--porcelain=v1")
        .output()
        .map_err(|e| {
            AppError::Config(format!(
                "Failed to execute git. Is it installed and in your PATH? Error: {e}"
            ))
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::Config(format!(
            "git status command failed: {stderr}"
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.trim().is_empty() {
        return Ok(());
    }

    let protection = PathProtection::new()?;
    let mut dirty_allowed_files = HashSet::new();

    for line in stdout.lines() {
        let path_part = &line[3..];
        if path_part.contains(" -> ") {
            let parts: Vec<&str> = path_part.split(" -> ").collect();
            let orig_path_str = parts[0];
            let new_path_str = parts[1];

            let orig_path = PathBuf::from(orig_path_str);
            if protection.validate(&orig_path).is_ok() {
                dirty_allowed_files.insert(orig_path_str.to_string());
            }

            let new_path = PathBuf::from(new_path_str);
            if protection.validate(&new_path).is_ok() {
                dirty_allowed_files.insert(new_path_str.to_string());
            }
        } else {
            let path_str = path_part;
            let path = PathBuf::from(path_str);
            if protection.validate(&path).is_ok() {
                dirty_allowed_files.insert(path_str.to_string());
            }
        }
    }

    if !dirty_allowed_files.is_empty() {
        let mut sorted_files: Vec<String> = dirty_allowed_files.into_iter().collect();
        sorted_files.sort();
        let file_list = sorted_files.join("\n- ");
        return Err(AppError::Config(format!(
            "Uncommitted changes found in files that can be modified by the LLM:\n- {file_list}\n\nPlease commit or stash your changes before running.\nTo override this safety check, use the --force or --f flag."
        )));
    }

    Ok(())
}
