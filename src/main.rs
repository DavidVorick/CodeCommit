mod app_error;
mod cli;
mod committing_code;
mod config;
#[cfg(test)]
mod config_test;
mod consistency;
mod context_builder;
mod init;
#[cfg(test)]
mod init_test;
mod llm;
mod logger;
mod system_prompts;

use crate::app_error::AppError;
use crate::cli::Workflow;
use std::process::exit;

#[tokio::main]
async fn main() {
    let result = run().await;

    match result {
        Ok(_) => {
            println!("Workflow completed successfully.");
            exit(0);
        }
        Err(e) => {
            eprintln!("An error occurred: {e}");
            exit(1);
        }
    }
}

async fn run() -> Result<(), AppError> {
    // Non-agentic 'init' command
    let args_vec: Vec<String> = std::env::args().skip(1).collect();
    if matches!(args_vec.first().map(|s| s.as_str()), Some("init")) {
        let project_name = args_vec.get(1).ok_or_else(|| {
            AppError::Config(
                "Please provide a project name: code-commit init <project-name>".to_string(),
            )
        })?;
        init::run_init_command(project_name)?;
        println!(
            "Place your gemini-key.txt and openai-key.txt files into the agent-config/ directory."
        );
        return Ok(());
    }

    let cli_args = cli::parse_cli_args()?;

    let logger_suffix = match cli_args.workflow {
        Workflow::CommitCode => "committing-code",
        Workflow::ConsistencyCheck => "consistency",
    };
    let logger = logger::Logger::new(logger_suffix)?;

    let result = match cli_args.workflow {
        Workflow::CommitCode => committing_code::run(&logger, cli_args).await,
        Workflow::ConsistencyCheck => consistency::run(&logger, cli_args).await,
    };

    if let Err(e) = &result {
        let _ = logger.log_text("final_error.txt", &e.to_string());
    }

    result
}
