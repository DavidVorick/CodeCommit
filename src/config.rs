use crate::app_error::AppError;
use crate::prompts::{INITIAL_QUERY_SYSTEM_PROMPT, REPAIR_QUERY_SYSTEM_PROMPT};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub struct Config {
    pub gemini_api_key: String,
    pub query: String,
    pub code_rollup: String,
}

impl Config {
    pub fn load() -> Result<Self, AppError> {
        let gemini_api_key = read_file_to_string("gemini-key.txt")?;
        let query = read_file_to_string("query.txt")?;
        let code_rollup = read_file_to_string("codeRollup.txt")?;

        Ok(Self {
            gemini_api_key: gemini_api_key.trim().to_string(),
            query,
            code_rollup,
        })
    }

    pub fn build_initial_prompt(&self) -> String {
        format!(
            "{}\n[query]\n{}\n[codebase]\n{}",
            INITIAL_QUERY_SYSTEM_PROMPT, self.query, self.code_rollup
        )
    }

    pub fn build_repair_prompt(
        &self,
        build_output: &str,
        file_replacements: &HashMap<PathBuf, Option<String>>,
    ) -> String {
        let replacements_str = format_file_replacements(file_replacements);
        format!(
            "{}\n[build.sh output]\n{}\n[query]\n{}\n[codebase]\n{}\n[file replacements]\n{}",
            REPAIR_QUERY_SYSTEM_PROMPT,
            build_output,
            self.query,
            self.code_rollup,
            replacements_str
        )
    }
}

fn format_file_replacements(replacements: &HashMap<PathBuf, Option<String>>) -> String {
    let mut result = String::new();
    let mut sorted_replacements: Vec<_> = replacements.iter().collect();
    sorted_replacements.sort_by_key(|(path, _)| (*path).clone());

    for (path, content_opt) in sorted_replacements {
        let path_str = path.to_string_lossy();
        match content_opt {
            Some(content) => {
                result.push_str(&format!("--- FILE REPLACEMENT {path_str} ---\n"));
                result.push_str(content);
                // Ensure a newline separates the content from the next header
                if !content.ends_with('\n') {
                    result.push('\n');
                }
                result.push('\n');
            }
            None => {
                result.push_str(&format!("--- FILE REMOVED {path_str} ---\n\n"));
            }
        }
    }
    result
}

fn read_file_to_string(path: impl AsRef<Path>) -> Result<String, AppError> {
    let path = path.as_ref();
    fs::read_to_string(path)
        .map_err(|e| AppError::Config(format!("Failed to read file '{}': {}", path.display(), e)))
}
