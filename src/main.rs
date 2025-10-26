mod app_error;
mod build_runner;
mod cli;
mod committing_code;
mod config;
mod consistency;
mod file_updater;
mod llm;
mod logger;
mod response_parser;
mod system_prompts;

#[cfg(test)]
mod cli_test;
#[cfg(test)]
mod file_updater_gitignore_tests;
#[cfg(test)]
mod file_updater_test;
#[cfg(test)]
mod response_parser_test;

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
