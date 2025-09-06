use crate::app_error::AppError;
use std::path::PathBuf;

#[derive(Debug, PartialEq)]
pub struct FileUpdate {
    pub path: PathBuf,
    pub content: String,
}

pub fn parse_llm_response(text: &str) -> Result<Vec<FileUpdate>, AppError> {
    let mut updates = Vec::new();
    let mut lines = text.lines();

    while let Some(line) = lines.next() {
        if line.starts_with("^^^") && !line.starts_with("^^^end") {
            let path_str = &line[3..];
            if path_str.is_empty() {
                return Err(AppError::ResponseParsing(
                    "Found '^^^' without a filename.".to_string(),
                ));
            }
            let path = PathBuf::from(path_str);

            let content_lines: Vec<&str> = lines
                .by_ref()
                .take_while(|&l| !l.starts_with("^^^end"))
                .collect();

            updates.push(FileUpdate {
                path,
                content: content_lines.join("\n"),
            });
        }
    }
    Ok(updates)
}
