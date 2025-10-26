use crate::app_error::AppError;
use crate::cli::{CliArgs, Model, Workflow};
use crate::system_prompts::{
    COMMITTING_CODE_INITIAL_QUERY, COMMITTING_CODE_REFACTOR_QUERY, CONSISTENCY_CHECK,
    PROJECT_STRUCTURE,
};
use std::fs;
use std::path::Path;

pub struct Config {
    pub model: Model,
    pub api_key: String,
    pub query: String,
    pub system_prompts: String,
}

impl Config {
    pub fn load(args: &CliArgs) -> Result<Self, AppError> {
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

        let system_prompts = match args.workflow {
            Workflow::CommitCode => {
                if args.refactor {
                    COMMITTING_CODE_REFACTOR_QUERY.to_string()
                } else {
                    COMMITTING_CODE_INITIAL_QUERY.to_string()
                }
            }
            Workflow::ConsistencyCheck => {
                format!("{PROJECT_STRUCTURE}\n{CONSISTENCY_CHECK}")
            }
        };

        Ok(Self {
            model: args.model,
            api_key: api_key.trim().to_string(),
            query,
            system_prompts,
        })
    }
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
