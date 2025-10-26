use crate::app_error::AppError;
use crate::build_runner;
use crate::cli::CliArgs;
use crate::config::Config;
use crate::file_updater;
use crate::llm;
use crate::logger;
use crate::response_parser;
use std::collections::HashMap;
use std::path::PathBuf;

const MAX_ATTEMPTS: u32 = 4;

pub async fn run(logger: &logger::Logger, cli_args: CliArgs) -> Result<(), AppError> {
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
