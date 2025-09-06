use crate::app_error::AppError;
use std::fs;
use std::path::Path;

pub struct Config {
    pub gemini_api_key: String,
    pub project_prompt: String,
    pub query: String,
    pub code_rollup: String,
    pub initial_query_system_prompt: String,
    pub repair_query_system_prompt: String,
}

impl Config {
    pub fn load() -> Result<Self, AppError> {
        let gemini_api_key = read_file_to_string("gemini-key.txt")?;
        let project_prompt = read_file_to_string("project-prompt.txt")?;
        let query = read_file_to_string("query.txt")?;
        let code_rollup = read_file_to_string("codeRollup.txt")?;

        let user_spec = read_file_to_string("UserSpecification.md")?;

        let initial_query_system_prompt = extract_prompt_section(
            &user_spec,
            "## Initial Query System Prompt",
            "## Repair Query System Prompt",
        )?;
        let repair_query_system_prompt =
            extract_prompt_section(&user_spec, "## Repair Query System Prompt", "## Logging")?;

        Ok(Self {
            gemini_api_key: gemini_api_key.trim().to_string(),
            project_prompt,
            query,
            code_rollup,
            initial_query_system_prompt,
            repair_query_system_prompt,
        })
    }

    pub fn build_initial_prompt(&self) -> String {
        format!(
            "{}\n{}\n{}\n{}",
            self.initial_query_system_prompt, self.project_prompt, self.query, self.code_rollup
        )
    }

    pub fn build_repair_prompt(&self, build_output: &str, broken_codebase: &str) -> String {
        format!(
            "{}\n[build script output]\n{}\n[query]\n{}\n[broken codebase]\n{}",
            self.repair_query_system_prompt, build_output, self.query, broken_codebase
        )
    }
}

fn read_file_to_string(path: impl AsRef<Path>) -> Result<String, AppError> {
    let path = path.as_ref();
    fs::read_to_string(path)
        .map_err(|e| AppError::Config(format!("Failed to read file '{}': {}", path.display(), e)))
}

fn extract_prompt_section(
    content: &str,
    start_marker: &str,
    end_marker: &str,
) -> Result<String, AppError> {
    let start = content
        .find(start_marker)
        .ok_or_else(|| AppError::Config(format!("Start marker '{start_marker}' not found")))?
        + start_marker.len();
    let end = content[start..]
        .find(end_marker)
        .ok_or_else(|| AppError::Config(format!("End marker '{end_marker}' not found")))?;

    Ok(content[start..start + end].trim().to_string())
}
