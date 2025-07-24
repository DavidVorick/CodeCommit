use path_clean::PathClean;
use std::collections::HashMap;
use std::path::{Component, PathBuf};

#[derive(Debug, Default)]
pub struct ParsedResponse {
    pub user_thoughts: Vec<String>,
    pub debug_thoughts: Vec<String>,
    pub file_changes: HashMap<PathBuf, String>,
    pub success_message: Option<String>,
}

fn extract_blocks(mut text: String, tag: &str) -> (Vec<(String, String)>, String) {
    let mut results = Vec::new();
    let start_tag_generic = format!("{}{}", tag, ""); // For filenames and 'start'
    let end_tag = format!("\n{}end", tag);

    while let Some(start_index) = text.find(&start_tag_generic) {
        let header_start = start_index + tag.len();
        let Some(header_end) = text[header_start..].find('\n') else { break; };
        let header = text[header_start..header_start + header_end].to_string();

        let content_start = header_start + header_end + 1;

        if let Some(end_index) = text[content_start..].find(&end_tag) {
            let content = text[content_start..content_start + end_index].to_string();
            results.push((header, content));

            text.replace_range(start_index..content_start + end_index + end_tag.len(), "");
        } else {
            break;
        }
    }
    (results, text)
}

fn validate_path(path_str: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(path_str);
    let cleaned_path = path.clean();

    for component in cleaned_path.components() {
        match component {
            Component::RootDir | Component::ParentDir => {
                return Err(format!(
                    "Path traversal ('..' or absolute paths) is not allowed: {}",
                    path_str
                ));
            }
            Component::Normal(part) => {
                if part.to_str() == Some(".git") {
                    return Err("Access to the .git directory is forbidden".to_string());
                }
            }
            _ => {}
        }
    }
    Ok(cleaned_path)
}

pub fn parse(text: &str) -> Result<ParsedResponse, String> {
    let mut response = ParsedResponse::default();
    let mut remaining_text = text.to_string();
    let mut temp_file_changes = HashMap::new();

    let (blocks, new_text) = extract_blocks(remaining_text, "&&&");
    remaining_text = new_text;
    for (header, content) in blocks {
        if header != "start" { return Err(format!("Invalid header for &&& block: {}", header)); }
        response.user_thoughts.push(content);
    }

    let (blocks, new_text) = extract_blocks(remaining_text, "%%%");
    remaining_text = new_text;
    for (header, content) in blocks {
         if header != "start" { return Err(format!("Invalid header for %%% block: {}", header)); }
        response.debug_thoughts.push(content);
    }

    let (blocks, new_text) = extract_blocks(remaining_text, "^^^");
    remaining_text = new_text;
    for (filename, content) in blocks {
        let path = validate_path(&filename)?;
        if temp_file_changes.insert(path, content).is_some() {
            return Err(format!("Duplicate file modification detected for: {}", filename));
        }
    }
    response.file_changes = temp_file_changes;

    let (blocks, new_text) = extract_blocks(remaining_text, "$$$");
    remaining_text = new_text;
    if !blocks.is_empty() {
         if blocks.len() > 1 { return Err("Multiple $$$ blocks found, only one is allowed.".to_string()); }
         let (header, content) = blocks.into_iter().next().unwrap();
         if header != "start" { return Err(format!("Invalid header for $$$ block: {}", header)); }
         response.success_message = Some(content);
    }

    if !remaining_text.trim().is_empty() {
        return Err(format!(
            "Malformed response: Unparsed content remains. Possible unterminated or nested blocks. Remainder: '{}'",
            remaining_text.chars().take(100).collect::<String>()
        ));
    }

    if response.file_changes.is_empty() && response.success_message.is_none() {
        return Err("Parsing error: Response must contain either file changes (^^^) or a success signal ($$$).".to_string());
    }

    if !response.file_changes.is_empty() && response.success_message.is_some() {
        return Err("Parsing error: Response cannot contain both file changes (^^^) and a success signal ($$$).".to_string());
    }

    Ok(response)
}

pub fn format_file_changes_for_prompt(file_changes: &HashMap<PathBuf, String>) -> String {
    if file_changes.is_empty() {
        return "No file changes were suggested.".to_string();
    }
    let mut s = String::new();
    for (path, content) in file_changes {
        s.push_str(&format!("^^^{}\n", path.display()));
        s.push_str(content);
        s.push_str("\n^^^end\n");
    }
    s
}