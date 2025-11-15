use crate::app_error::AppError;
use crate::cli::{CliArgs, Model, Workflow};
use crate::system_prompts::{COMMITTING_CODE_INITIAL_QUERY, CONSISTENCY_CHECK, PROJECT_STRUCTURE};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Config {
    pub model: Model,
    pub api_key: String,
    pub query: String,
    pub system_prompts: String,
}

impl Config {
    pub fn load(args: &CliArgs) -> Result<Self, AppError> {
        Self::load_from_dir(args, Path::new("."))
    }

    pub fn load_from_dir(args: &CliArgs, base_dir: &Path) -> Result<Self, AppError> {
        check_gitignore_in_dir(base_dir)?;

        match args.workflow {
            Workflow::CommitCode | Workflow::ConsistencyCheck => {
                let api_key_rel = match args.model {
                    Model::Gemini2_5Pro => PathBuf::from("agent-config/gemini-key.txt"),
                    Model::Gpt5 => PathBuf::from("agent-config/openai-key.txt"),
                };
                let api_key = read_file_to_string_at(base_dir, &api_key_rel)?;

                let query_rel = PathBuf::from("agent-config/query.txt");
                let query = match args.workflow {
                    Workflow::CommitCode => read_file_to_string_at(base_dir, &query_rel)?,
                    Workflow::ConsistencyCheck => {
                        let path = base_dir.join(&query_rel);
                        match fs::read_to_string(&path) {
                            Ok(content) => content,
                            Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
                            Err(e) => {
                                return Err(AppError::Config(format!(
                                    "Failed to read file '{}': {}",
                                    query_rel.display(),
                                    e
                                )));
                            }
                        }
                    }
                    Workflow::Rollup => unreachable!(),
                };

                let system_prompts = match args.workflow {
                    Workflow::CommitCode => COMMITTING_CODE_INITIAL_QUERY.to_string(),
                    Workflow::ConsistencyCheck => {
                        format!("{PROJECT_STRUCTURE}\n{CONSISTENCY_CHECK}")
                    }
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
