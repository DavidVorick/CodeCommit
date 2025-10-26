use crate::app_error::AppError;
use std::path::PathBuf;

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct FileUpdate {
    pub path: PathBuf,
    pub content: Option<String>,
}

pub(crate) fn parse_llm_response(text: &str) -> Result<Vec<FileUpdate>, AppError> {
    let mut updates = Vec::new();
    let mut lines = text.lines().peekable();

    while let Some(line) = lines.next() {
        if line.starts_with("^^^") && line != "^^^end" && line != "^^^delete" {
            let path_str = &line[3..];
            if path_str.is_empty() {
                return Err(AppError::ResponseParsing(
                    "Found '^^^' without a filename.".to_string(),
                ));
            }
            let path = PathBuf::from(path_str);

            if lines.peek() == Some(&"^^^delete") {
                lines.next();
                updates.push(FileUpdate {
                    path,
                    content: None,
                });
                continue;
            }

            let content_lines: Vec<&str> = lines.by_ref().take_while(|&l| l != "^^^end").collect();

            updates.push(FileUpdate {
                path,
                content: Some(content_lines.join("\n")),
            });
        }
    }
    Ok(updates)
}

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
