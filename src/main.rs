mod app_error;
mod build_runner;
mod cli;
mod config;
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
use crate::cli::{CliArgs, Workflow};
use crate::config::Config;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::exit;

const MAX_ATTEMPTS: u32 = 4;

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
        Workflow::CommitCode => run_commit_code(&logger, cli_args).await,
        Workflow::ConsistencyCheck => run_consistency_check(&logger, cli_args).await,
    };

    if let Err(e) = &result {
        let _ = logger.log_text("final_error.txt", &e.to_string());
    }

    result
}

async fn run_iterative_workflow(
    logger: &logger::Logger,
    cli_args: CliArgs,
) -> Result<(), AppError> {
    let config = Config::load(cli_args)?;

    let mut last_build_output: Option<String> = None;
    let mut cumulative_updates: HashMap<PathBuf, Option<String>> = HashMap::new();

    for attempt in 1..=MAX_ATTEMPTS {
        println!("Starting attempt {attempt}/{MAX_ATTEMPTS}...");

        let (prompt, name_part) = if attempt == 1 {
            (config.build_initial_prompt(), "initial-query")
        } else {
            let build_output = last_build_output
                .as_ref()
                .expect("Build output should exist for repair attempts");
            (
                config.build_repair_prompt(build_output, &cumulative_updates),
                "repair",
            )
        };
        let log_prefix = format!("{attempt}-{name_part}");

        let response_text = llm::query(
            config.model,
            config.api_key.clone(),
            &prompt,
            logger,
            &log_prefix,
        )
        .await?;

        println!("Parsing LLM response and applying file updates...");
        let updates = response_parser::parse_llm_response(&response_text)?;

        for update in &updates {
            cumulative_updates.insert(update.path.clone(), update.content.clone());
        }

        file_updater::apply_updates(&updates)?;

        println!("Running build script...");
        match build_runner::run() {
            Ok(output) => {
                logger.log_text(&format!("{log_prefix}-build.txt"), &output)?;
                println!("Build successful!");
                return Ok(());
            }
            Err(build_failure) => {
                logger.log_text(&format!("{log_prefix}-build.txt"), &build_failure.output)?;
                println!("Build failed. Preparing for repair attempt...");
                last_build_output = Some(build_failure.output);
            }
        }
    }

    println!("Build did not pass after {MAX_ATTEMPTS} attempts. Aborting.");
    Err(AppError::MaxAttemptsReached)
}

async fn run_commit_code(logger: &logger::Logger, cli_args: CliArgs) -> Result<(), AppError> {
    run_iterative_workflow(logger, cli_args).await
}

async fn run_consistency_check(logger: &logger::Logger, cli_args: CliArgs) -> Result<(), AppError> {
    println!("Starting consistency check workflow...");
    let config = Config::load(cli_args)?;

    let prompt = config.build_initial_prompt();
    let log_prefix = "1-consistency-check";

    let response_text = llm::query(
        config.model,
        config.api_key.clone(),
        &prompt,
        logger,
        log_prefix,
    )
    .await?;

    println!("Writing consistency report...");
    let report_dir = PathBuf::from("agent-config");
    fs::create_dir_all(&report_dir)?;
    let report_path = report_dir.join("consistency-report.txt");
    fs::write(&report_path, response_text)?;

    println!("Consistency report written to {}", report_path.display());

    Ok(())
}
