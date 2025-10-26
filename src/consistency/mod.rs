use crate::app_error::AppError;
use crate::cli::CliArgs;
use crate::config::Config;
use crate::llm;
use crate::logger;
use std::fs;
use std::path::PathBuf;

pub async fn run(logger: &logger::Logger, cli_args: CliArgs) -> Result<(), AppError> {
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
