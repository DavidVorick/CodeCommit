use crate::app_error::AppError;
use crate::cli::{CliArgs, Model};
use crate::config::Config;
use crate::context_builder;
use crate::llm;
use crate::logger::Logger;
use crate::system_prompts;
use std::future::Future;
use std::pin::Pin;

pub trait ConsistencyDeps {
    fn build_context<'a>(
        &'a self,
        prompt: &'a str,
        config: &'a Config,
        logger: &'a Logger,
        prefix: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<String, AppError>> + Send + 'a>>;

    fn query_llm<'a>(
        &'a self,
        model: Model,
        api_key: String,
        prompt: &'a str,
        logger: &'a Logger,
        prefix: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<String, AppError>> + Send + 'a>>;
}

struct RealDeps;

impl ConsistencyDeps for RealDeps {
    fn build_context<'a>(
        &'a self,
        prompt: &'a str,
        config: &'a Config,
        logger: &'a Logger,
        prefix: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<String, AppError>> + Send + 'a>> {
        Box::pin(context_builder::build_codebase_context(
            prompt, config, logger, prefix,
        ))
    }

    fn query_llm<'a>(
        &'a self,
        model: Model,
        api_key: String,
        prompt: &'a str,
        logger: &'a Logger,
        prefix: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<String, AppError>> + Send + 'a>> {
        Box::pin(llm::query(model, api_key, prompt, logger, prefix))
    }
}

pub async fn run(logger: &Logger, cli_args: CliArgs) -> Result<(), AppError> {
    let config = Config::load(&cli_args)?;
    run_internal(logger, config, &RealDeps).await
}

async fn run_internal(
    logger: &Logger,
    config: Config,
    deps: &impl ConsistencyDeps,
) -> Result<(), AppError> {
    println!("Building codebase context for consistency check...");
    let next_agent_prompt = format!(
        "{}\n{}\n[supervisor query]\n{}",
        system_prompts::PROJECT_STRUCTURE,
        system_prompts::CONSISTENCY_CHECK,
        config.query
    );

    let codebase = deps
        .build_context(&next_agent_prompt, &config, logger, "1-consistency-context")
        .await?;
    logger.log_text("codebase_for_consistency.txt", &codebase)?;

    println!("Running consistency check...");
    let prompt = format!("{next_agent_prompt}\n[codebase]\n{codebase}");

    let report = deps
        .query_llm(
            config.model,
            config.api_key.clone(),
            &prompt,
            logger,
            "2-consistency",
        )
        .await?;

    println!("\n{report}");

    Ok(())
}

#[cfg(test)]
#[path = "mod_test.rs"]
mod tests;
