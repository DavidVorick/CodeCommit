use crate::app_error::AppError;
use crate::cli::{CliArgs, Model, Workflow};
use crate::system_prompts::{
    CODE_MODIFICATION_INSTRUCTIONS, COMMITTING_CODE_INITIAL_QUERY, COMMITTING_CODE_REFACTOR_QUERY,
    COMMITTING_CODE_REPAIR_QUERY, CONSISTENCY_CHECK, PROJECT_STRUCTURE,
};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub struct Config {
    pub model: Model,
    pub api_key: String,
    pub query: String,
    pub code_rollup: String,
    pub workflow: Workflow,
    pub refactor: bool,
}

impl Config {
    pub fn load(args: CliArgs) -> Result<Self, AppError> {
        check_gitignore()?;

        let api_key_path = match args.model {
            Model::Gemini2_5Pro => "agent-config/gemini-key.txt",
            Model::Gpt5 => "agent-config/openai-key.txt",
        };
        let api_key = read_file_to_string(api_key_path)?;

        let query = match args.workflow {
            Workflow::CommitCode => read_file_to_string("agent-config/query.txt")?,
            Workflow::ConsistencyCheck => {
                let path = Path::new("agent-config/consistency-query.txt");
                match fs::read_to_string(path) {
                    Ok(content) => content,
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
                    Err(e) => {
                        return Err(AppError::Config(format!(
                            "Failed to read file '{}': {}",
                            path.display(),
                            e
                        )));
                    }
                }
            }
        };

        let code_rollup = read_file_to_string("agent-config/codeRollup.txt")?;

        Ok(Self {
            model: args.model,
            api_key: api_key.trim().to_string(),
            query,
            code_rollup,
            workflow: args.workflow,
            refactor: args.refactor,
        })
    }

    pub fn build_initial_prompt(&self) -> String {
        let system_prompt = match self.workflow {
            Workflow::CommitCode => {
                let initial_query_prompt = if self.refactor {
                    COMMITTING_CODE_REFACTOR_QUERY
                } else {
                    COMMITTING_CODE_INITIAL_QUERY
                };
                format!(
                    "{PROJECT_STRUCTURE}\n{CODE_MODIFICATION_INSTRUCTIONS}\n{initial_query_prompt}"
                )
            }
            Workflow::ConsistencyCheck => {
                format!("{PROJECT_STRUCTURE}\n{CONSISTENCY_CHECK}")
            }
        };
        format!(
            "{}\n[query]\n{}\n[codebase]\n{}",
            system_prompt, self.query, self.code_rollup
        )
    }

    pub fn build_repair_prompt(
        &self,
        build_output: &str,
        file_replacements: &HashMap<PathBuf, Option<String>>,
    ) -> String {
        let replacements_str = format_file_replacements(file_replacements);
        let system_prompt = match self.workflow {
            Workflow::CommitCode => format!(
                "{PROJECT_STRUCTURE}\n{CODE_MODIFICATION_INSTRUCTIONS}\n{COMMITTING_CODE_REPAIR_QUERY}"
            ),
            Workflow::ConsistencyCheck => {
                panic!("Consistency check workflow does not support repair prompts.")
            }
        };
        format!(
            "{}\n[build.sh output]\n{}\n[query]\n{}\n[codebase]\n{}\n[file replacements]\n{}",
            system_prompt, build_output, self.query, self.code_rollup, replacements_str
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
                "'.gitignore' file not found. It must exist and contain '/agent-config' to protect secrets.".to_string()
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

    if !gitignore_content.lines().any(|line| {
        let trimmed = line.trim();
        trimmed == "/agent-config" || trimmed == "agent-config/"
    }) {
        return Err(AppError::Config(
            "Security check failed: Your .gitignore file must contain the line '/agent-config' to prevent accidental exposure of your API keys and logs.".to_string()
        ));
    }

    Ok(())
}
