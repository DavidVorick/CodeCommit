use crate::app_error::AppError;
use crate::cli::{CliArgs, Model};
use crate::prompts::{INITIAL_QUERY_SYSTEM_PROMPT, REPAIR_QUERY_SYSTEM_PROMPT};
use crate::prompts_consistency::CONSISTENCY_CHECK_SYSTEM_PROMPT;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub struct Config {
    pub model: Model,
    pub api_key: String,
    pub query: String,
    pub code_rollup: String,
}

impl Config {
    pub fn load(args: CliArgs) -> Result<Self, AppError> {
        check_gitignore()?;

        let api_key_path = match args.model {
            Model::Gemini2_5Pro => "agent-config/gemini-key.txt",
            Model::Gpt5 => "agent-config/openai-key.txt",
        };
        let api_key = read_file_to_string(api_key_path)?;

        let query = read_file_to_string("agent-config/query.txt")?;
        let code_rollup = read_file_to_string("agent-config/codeRollup.txt")?;

        Ok(Self {
            model: args.model,
            api_key: api_key.trim().to_string(),
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

    pub fn build_consistency_prompt(&self) -> String {
        format!(
            "{}\n[query]\n{}\n[codebase]\n{}",
            CONSISTENCY_CHECK_SYSTEM_PROMPT, self.query, self.code_rollup
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

fn check_gitignore() -> Result<(), AppError> {
    let gitignore_path = Path::new(".gitignore");
    let gitignore_content = match fs::read_to_string(gitignore_path) {
        Ok(content) => content,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(AppError::Config(
                "'.gitignore' file not found. Please ensure key files like '/agent-config/gemini-key.txt' are listed in it, or the whole '/agent-config' directory.".to_string()
            ));
        }
        Err(e) => {
            return Err(AppError::Config(format!(
                "Failed to read file '{}': {}",
                gitignore_path.display(),
                e
            )));
        }
    };

    // If the whole directory is ignored, we're good.
    if gitignore_content.lines().any(|line| {
        let trimmed = line.trim();
        trimmed == "/agent-config" || trimmed == "agent-config/"
    }) {
        return Ok(());
    }

    // Otherwise, check for individual key files.
    let required_entries = [
        "/agent-config/gemini-key.txt",
        "/agent-config/openai-key.txt",
    ];
    for entry in required_entries {
        if !gitignore_content.lines().any(|line| line.trim() == entry) {
            return Err(AppError::Config(
                format!("Security check failed: Your .gitignore file must contain the line '{entry}' or ignore '/agent-config' to prevent accidental exposure of your API key.")
            ));
        }
    }

    Ok(())
}
