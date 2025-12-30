use crate::app_error::AppError;
use crate::cli::{CliArgs, Model, Workflow};
use crate::system_prompts::{COMMITTING_CODE_INITIAL_QUERY, CONSISTENCY_CHECK, PROJECT_STRUCTURE};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

#[cfg(test)]
mod config_test;

#[derive(Debug)]
pub struct Config {
    pub model: Model,
    pub api_key: String,
    pub query: String,
    pub system_prompts: String,
}

impl Config {
    pub fn load(args: &CliArgs) -> Result<Self, AppError> {
        let query = match args.workflow {
            Workflow::CommitCode | Workflow::ConsistencyCheck => Self::get_query_from_editor()?,
            Workflow::Rollup | Workflow::Auto => String::new(),
        };

        Self::load_from_dir(args, Path::new("."), query)
    }

    pub(crate) fn get_query_from_editor() -> Result<String, AppError> {
        let editor = std::env::var("VISUAL")
            .or_else(|_| std::env::var("EDITOR"))
            .unwrap_or_else(|_| "vi".to_string());

        let file = tempfile::Builder::new()
            .suffix(".md")
            .tempfile()
            .map_err(AppError::Io)?;

        let file_path = file.path();

        let status = Command::new(&editor)
            .arg(file_path)
            .status()
            .map_err(|e| AppError::Config(format!("Failed to launch editor '{editor}': {e}")))?;

        if !status.success() {
            return Err(AppError::Config(
                "Editor exited with non-zero status.".to_string(),
            ));
        }

        let mut buffer = String::new();
        // Open file path freshly to handle atomic saves by editors
        let mut f = fs::File::open(file_path).map_err(AppError::Io)?;
        f.read_to_string(&mut buffer).map_err(AppError::Io)?;

        let query = buffer.trim().to_string();
        Ok(query)
    }

    pub fn load_from_dir(args: &CliArgs, base_dir: &Path, query: String) -> Result<Self, AppError> {
        check_gitignore_in_dir(base_dir)?;

        match args.workflow {
            Workflow::CommitCode | Workflow::ConsistencyCheck | Workflow::Auto => {
                let api_key_rel = match args.model {
                    Model::Gemini3Pro | Model::Gemini2_5Pro => {
                        PathBuf::from("agent-config/gemini-key.txt")
                    }
                    Model::Gpt5 => PathBuf::from("agent-config/openai-key.txt"),
                };
                let api_key = read_file_to_string_at(base_dir, &api_key_rel)?;

                let system_prompts = match args.workflow {
                    Workflow::CommitCode => COMMITTING_CODE_INITIAL_QUERY.to_string(),
                    Workflow::ConsistencyCheck => {
                        format!("{PROJECT_STRUCTURE}\n{CONSISTENCY_CHECK}")
                    }
                    Workflow::Auto => String::new(), // Prompts handled internally
                    Workflow::Rollup => unreachable!(),
                };

                Ok(Self {
                    model: args.model,
                    api_key: api_key.trim().to_string(),
                    query,
                    system_prompts,
                })
            }
            Workflow::Rollup => Err(AppError::Config(
                "The rollup workflow does not require configuration.".to_string(),
            )),
        }
    }
}

fn read_file_to_string_at(base_dir: &Path, rel_path: &Path) -> Result<String, AppError> {
    let full_path = base_dir.join(rel_path);
    fs::read_to_string(&full_path).map_err(|e| {
        AppError::Config(format!(
            "Failed to read file '{}': {}",
            rel_path.display(),
            e
        ))
    })
}

fn check_gitignore_in_dir(base_dir: &Path) -> Result<(), AppError> {
    let gitignore_rel = PathBuf::from(".gitignore");
    let gitignore_path = base_dir.join(&gitignore_rel);
    let gitignore_content = match fs::read_to_string(&gitignore_path) {
        Ok(content) => content,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(AppError::Config(
                "'.gitignore' file not found. It must exist and contain '/agent-config' to protect secrets.".to_string()
            ));
        }
        Err(e) => {
            return Err(AppError::Config(format!(
                "Failed to read file '{}': {}",
                gitignore_rel.display(),
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
