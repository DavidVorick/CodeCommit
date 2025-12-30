use crate::app_error::AppError;
use crate::cli::CliArgs;
use crate::config::Config;
use crate::context_builder;
use crate::llm;
use crate::logger::Logger;

pub async fn run(logger: &Logger, cli_args: CliArgs) -> Result<(), AppError> {
    let config = Config::load(&cli_args)?;

    println!("Building codebase context for consistency check...");
    let next_agent_prompt = format!(
        "{}\n[supervisor query]\n{}",
        config.system_prompts, config.query
    );
    let codebase = context_builder::build_codebase_context(
        &next_agent_prompt,
        &config,
        logger,
        "consistency-context-query",
    )
    .await?;
    logger.log_text("codebase_for_consistency.txt", &codebase)?;

    println!("Running consistency check...");
    let prompt = format!("{next_agent_prompt}\n[codebase]\n{codebase}");

    let report = llm::query(
        config.model,
        config.api_key.clone(),
        &prompt,
        logger,
        "consistency-check",
    )
    .await?;

    println!("\n{report}");

    Ok(())
}
