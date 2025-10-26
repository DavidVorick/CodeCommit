use crate::app_error::AppError;
use std::path::PathBuf;

pub(crate) fn parse_context_llm_response(text: &str) -> Result<Vec<PathBuf>, AppError> {
    let mut in_files_block = false;
    let mut files = Vec::new();
    for line in text.lines() {
        if line.trim() == "%%%files" {
            in_files_block = true;
            continue;
        }
        if line.trim() == "%%%end" {
            if in_files_block {
                return Ok(files);
            } else {
                return Err(AppError::ResponseParsing(
                    "Found '%%%end' without a preceding '%%%files'.".to_string(),
                ));
            }
        }
        if in_files_block {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                files.push(PathBuf::from(trimmed));
            }
        }
    }
    if in_files_block {
        Err(AppError::ResponseParsing(
            "Found '%%%files' but no matching '%%%end'.".to_string(),
        ))
    } else {
        Err(AppError::ResponseParsing(
            "Could not find '%%%files'...'%%%end' block in context LLM response.".to_string(),
        ))
    }
}
