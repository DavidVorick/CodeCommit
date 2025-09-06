use crate::app_error::AppError;
use crate::prompts::{INITIAL_QUERY_SYSTEM_PROMPT, REPAIR_QUERY_SYSTEM_PROMPT};
use std::fs;
use std::path::Path;

pub struct Config {
    pub gemini_api_key: String,
    pub project_prompt: String,
    pub query: String,
    pub code_rollup: String,
}

impl Config {
    pub fn load() -> Result<Self, AppError> {
        let gemini_api_key = read_file_to_string("gemini-key.txt")?;
        let project_prompt = read_file_to_string("project-prompt.txt")?;
        let query = read_file_to_string("query.txt")?;
        let code_rollup = read_file_to_string("codeRollup.txt")?;

        Ok(Self {
            gemini_api_key: gemini_api_key.trim().to_string(),
            project_prompt,
            query,
            code_rollup,
        })
    }

    pub fn build_initial_prompt(&self) -> String {
        format!(
            "{}\n{}\n{}\n{}",
            INITIAL_QUERY_SYSTEM_PROMPT, self.project_prompt, self.query, self.code_rollup
        )
    }

    pub fn build_repair_prompt(&self, build_output: &str, broken_codebase: &str) -> String {
        format!(
            "{}\n[build script output]\n{}\n[query]\n{}\n[broken codebase]\n{}",
            REPAIR_QUERY_SYSTEM_PROMPT, build_output, self.query, broken_codebase
        )
    }
}

fn read_file_to_string(path: impl AsRef<Path>) -> Result<String, AppError> {
    let path = path.as_ref();
    fs::read_to_string(path)
        .map_err(|e| AppError::Config(format!("Failed to read file '{}': {}", path.display(), e)))
}
