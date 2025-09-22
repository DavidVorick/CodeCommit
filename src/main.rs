mod app_error;
mod build_runner;
mod cli;
mod config;
mod file_updater;
mod llm_api;
mod logger;
mod prompts;
mod response_parser;

#[cfg(test)]
mod file_updater_gitignore_tests;
#[cfg(test)]
mod file_updater_test;
#[cfg(test)]
mod llm_api_test;
#[cfg(test)]
mod response_parser_test;

use crate::app_error::AppError;
use crate::cli::Model;
use crate::config::Config;
use crate::llm_api::{GeminiClient, GptClient, LlmApiClient};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::exit;

// 1 initial attempt + 3 repair attempts
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
    // Create logger early. If this fails, we can't log, so we just propagate the error.
    let logger = logger::Logger::new()?;

    let result = run_internal(&logger).await;

    if let Err(e) = &result {
        // If the main loop fails, log the final error.
        let _ = logger.log_final_error(e);
    }

    result
}

async fn run_internal(logger: &logger::Logger) -> Result<(), AppError> {
    let config = Config::load()?;
    let llm_client = match config.model {
        Model::Gemini2_5Pro => LlmApiClient::Gemini(GeminiClient::new(config.api_key.clone())),
        Model::Gpt5 => LlmApiClient::Gpt(GptClient::new(config.api_key.clone())),
    };

    let mut last_build_output: Option<String> = None;
    // Track the cumulative file updates across all attempts.
    let mut cumulative_updates: HashMap<PathBuf, Option<String>> = HashMap::new();

    for attempt in 1..=MAX_ATTEMPTS {
        println!("Starting attempt {attempt}/{MAX_ATTEMPTS}...");

        let (prompt, log_name) = if attempt == 1 {
            (config.build_initial_prompt(), "initial-query".to_string())
        } else {
            let build_output = last_build_output
                .as_ref()
                .expect("Build output should exist for repair attempts");
            (
                config.build_repair_prompt(build_output, &cumulative_updates),
                format!("repair-query-{}", attempt - 1),
            )
        };
        logger.log_prompt(&log_name, &prompt)?;

        let response_json = match llm_client.query(&prompt).await {
            Ok(json) => json,
            Err(e) => {
                let error_msg = format!("ERROR\n{e}");
                logger.log_response_text(&log_name, &error_msg)?;
                return Err(e);
            }
        };
        logger.log_response_json(&log_name, &response_json)?;

        let response_text = match llm_client.extract_text_from_response(&response_json) {
            Ok(text) => text,
            Err(e) => {
                let error_msg = format!("ERROR\n{e}");
                logger.log_response_text(&log_name, &error_msg)?;
                return Err(e);
            }
        };
        logger.log_response_text(&log_name, &response_text)?;

        println!("Parsing LLM response and applying file updates...");
        let updates = response_parser::parse_llm_response(&response_text)?;

        // Update the cumulative list of changes for the next repair prompt.
        for update in &updates {
            cumulative_updates.insert(update.path.clone(), update.content.clone());
        }

        file_updater::apply_updates(&updates)?;

        println!("Running build script...");
        match build_runner::run() {
            Ok(output) => {
                logger.log_build_output(&log_name, &output)?;
                println!("Build successful!");
                return Ok(());
            }
            Err(build_failure) => {
                logger.log_build_output(&log_name, &build_failure.output)?;
                println!("Build failed. Preparing for repair attempt...");
                last_build_output = Some(build_failure.output);
            }
        }
    }

    println!("Build did not pass after {MAX_ATTEMPTS} attempts. Aborting.");
    Err(AppError::MaxAttemptsReached)
}
