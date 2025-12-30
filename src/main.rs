mod app_error;
mod auto_workflow;
mod cli;
mod committing_code;
mod config;
mod consistency;
mod context_builder;
mod init;
mod llm;
mod logger;
mod rollup;
mod system_prompts;

use app_error::AppError;
use logger::Logger;
use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::FAILURE
        }
    }
}

async fn run() -> Result<(), AppError> {
    let args = cli::parse_cli_args()?;

    let suffix = match &args.workflow {
        cli::Workflow::CommitCode => "committing-code",
        cli::Workflow::ConsistencyCheck => "consistency",
        cli::Workflow::Rollup => "rollup",
        cli::Workflow::Auto => "auto-workflow",
        cli::Workflow::Init(_) => "init",
    };

    let logger = Logger::new(suffix)?;

    match args.workflow {
        cli::Workflow::CommitCode => {
            committing_code::run(&logger, args).await?;
        }
        cli::Workflow::ConsistencyCheck => {
            consistency::run(&logger, args).await?;
        }
        cli::Workflow::Rollup => {
            rollup::run(&logger, args).await?;
        }
        cli::Workflow::Auto => {
            auto_workflow::run(&logger, args).await?;
        }
        cli::Workflow::Init(ref name) => {
            init::run_init_command(name)?;
        }
    }

    Ok(())
}
