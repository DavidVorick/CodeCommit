use crate::app_error::AppError;
use crate::cli::CliArgs;
use crate::config::Config;
use crate::context_builder;
use crate::llm;
use crate::logger::Logger;
use crate::system_prompts::{CONSISTENCY_CHECK, PROJECT_STRUCTURE};

pub async fn run(logger: &Logger, cli_args: CliArgs) -> Result<(), AppError> {
    let config = Config::load(&cli_args)?;

    println!("Building codebase context for consistency check...");
    let codebase = context_builder::build_codebase_context(&config, logger).await?;
    logger.log_text("codebase_for_consistency.txt", &codebase)?;

    println!("Running consistency check...");
    let prompt = format!(
        "{}\n{}\n[user query]\n{}\n[codebase]\n{}",
        PROJECT_STRUCTURE, CONSISTENCY_CHECK, config.query, codebase
    );

    let report = llm::query(
        config.model,
        config.api_key.clone(),
        &prompt,
        logger,
        "consistency-check",
    )
    .await?;

    println!("\nConsistency Check Report:\n");
    println!("{report}");

    Ok(())
}
