use crate::app_error::AppError;
use std::fs;
use std::path::Path;

pub fn apply_file_updates(response: &str) -> Result<(), AppError> {
    let mut current_pos = 0;
    while let Some(start_idx) = response[current_pos..].find("^^^") {
        let absolute_start = current_pos + start_idx;
        let rest = &response[absolute_start + 3..];

        // Find the newline after the filename
        if let Some(newline_idx) = rest.find('\n') {
            let filename = rest[..newline_idx].trim();
            let content_start = absolute_start + 3 + newline_idx + 1;

            // Find end tag
            if let Some(end_idx) = response[content_start..].find("^^^end") {
                let content = &response[content_start..content_start + end_idx];

                // Check if it is a delete command
                if content.trim() == "^^^delete" {
                    delete_file(filename)?;
                } else {
                    write_file(filename, content)?;
                }

                current_pos = content_start + end_idx + 6;
                continue;
            }
        }
        // If parsing fails, move forward to avoid infinite loop
        current_pos += 3;
    }
    Ok(())
}

fn write_file(filename: &str, content: &str) -> Result<(), AppError> {
    let path = Path::new(filename);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            AppError::FileUpdate(format!("Failed to create directories for {filename}: {e}"))
        })?;
    }

    fs::write(path, content)
        .map_err(|e| AppError::FileUpdate(format!("Failed to write file {filename}: {e}")))?;

    println!("Updated file: {filename}");
    Ok(())
}

fn delete_file(filename: &str) -> Result<(), AppError> {
    let path = Path::new(filename);
    if path.exists() {
        fs::remove_file(path)
            .map_err(|e| AppError::FileUpdate(format!("Failed to delete file {filename}: {e}")))?;
        println!("Deleted file: {filename}");
    }
    Ok(())
}
