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

pub(crate) fn parse_extra_files_response(text: &str) -> Result<Vec<PathBuf>, AppError> {
    let mut files = Vec::new();
    let lines = text.lines();
    let mut inside_block = false;

    for line in lines {
        let trimmed = line.trim();
        if trimmed == "%%%files" {
            inside_block = true;
            continue;
        }
        if trimmed == "%%%end" {
            inside_block = false;
            continue;
        }
        if inside_block && !trimmed.is_empty() {
            files.push(PathBuf::from(trimmed));
        }
    }
    Ok(files)
}
