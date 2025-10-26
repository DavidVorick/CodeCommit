use crate::app_error::AppError;
use ignore::WalkBuilder;
use std::fs;

pub fn assemble_codebase() -> Result<String, AppError> {
    let mut content = String::new();
    let walker = WalkBuilder::new("./").build();

    let mut paths = Vec::new();
    for result in walker {
        let entry =
            result.map_err(|e| AppError::Config(format!("Error walking directory: {e}")))?;
        if entry.file_type().is_some_and(|ft| ft.is_dir()) {
            continue;
        }
        paths.push(entry.into_path());
    }
    paths.sort();

    for path in paths {
        if path.file_name().is_some_and(|name| name == "code-commit") {
            continue;
        }

        if let Ok(file_content) = fs::read_to_string(&path) {
            content.push_str(&format!("--- {} ---\n", path.display()));
            content.push_str(&file_content);
            if !file_content.ends_with('\n') {
                content.push('\n');
            }
            content.push('\n');
        }
    }

    Ok(content)
}
