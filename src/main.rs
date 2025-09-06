mod app_error;
mod build_runner;
mod config;
mod file_updater;
mod llm_api;
mod logger;
mod response_parser;

#[cfg(test)]
mod response_parser_test;

use crate::app_error::AppError;
use crate::config::Config;
use crate::llm_api::GeminiClient;
use std::process::exit;

const MAX_ATTEMPTS: u32 = 3;

#[tokio::main]
async fn main() {
    match run().await {
        Ok(_) => {
            println!("✅ Workflow completed successfully.");
            exit(0);
        }
        Err(e) => {
            eprintln!("❌ An error occurred: {e}");
            exit(1);
        }
    }
}

async fn run() -> Result<(), AppError> {
    let config = Config::load()?;
    let logger = logger::Logger::new()?;
    let gemini_client = GeminiClient::new(config.gemini_api_key.clone());

    let mut last_build_output = String::new();
    let mut current_codebase = config.code_rollup.clone();

    for attempt in 1..=MAX_ATTEMPTS {
        println!("➡️ Starting attempt {attempt}/{MAX_ATTEMPTS}...");

        let prompt = if attempt == 1 {
            config.build_initial_prompt()
        } else {
            config.build_repair_prompt(&last_build_output, &current_codebase)
        };
        logger.log_prompt(attempt, &prompt)?;

        let response_json = gemini_client.query(&prompt).await?;
        logger.log_response_json(attempt, &response_json)?;

        let response_text = llm_api::extract_text_from_response(&response_json)?;
        logger.log_response_text(attempt, &response_text)?;

        println!("📝 Parsing LLM response and applying file updates...");
        let updates = response_parser::parse_llm_response(&response_text)?;
        file_updater::apply_updates(&updates)?;

        println!("🔨 Running build script...");
        match build_runner::run() {
            Ok(output) => {
                logger.log_build_output(attempt, &output)?;
                println!("✅ Build successful!");
                return Ok(());
            }
            Err(build_failure) => {
                logger.log_build_output(attempt, &build_failure.output)?;
                println!("⚠️ Build failed. Preparing for repair attempt...");
                last_build_output = build_failure.output;
                current_codebase = build_runner::get_codebase_rollup()?;
            }
        }
    }

    println!("❌ Build did not pass after {MAX_ATTEMPTS} attempts. Aborting.");
    Err(AppError::MaxAttemptsReached)
}
