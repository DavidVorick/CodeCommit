use crate::app_error::AppError;
use std::path::PathBuf;

#[derive(Debug, PartialEq, Clone)]
pub struct FileUpdate {
    pub path: PathBuf,
    /// The content to write. `Some(content)` for file creation/replacement
    /// (including empty files with `Some("")`), and `None` for deletion.
    pub content: Option<String>,
}

pub fn parse_llm_response(text: &str) -> Result<Vec<FileUpdate>, AppError> {
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

            // Check for the `^^^delete` marker immediately following the filename.
            if lines.peek() == Some(&"^^^delete") {
                lines.next(); // Consume '^^^delete'
                updates.push(FileUpdate {
                    path,
                    content: None,
                });
                continue;
            }

            // Otherwise, collect lines until `^^^end`.
            let content_lines: Vec<&str> = lines.by_ref().take_while(|&l| l != "^^^end").collect();

            updates.push(FileUpdate {
                path,
                content: Some(content_lines.join("\n")),
            });
        }
    }
    Ok(updates)
}
