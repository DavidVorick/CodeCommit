mod build_runner;
mod context_builder;
mod file_updater;
mod response_parser;

#[cfg(test)]
mod file_updater_gitignore_tests;
#[cfg(test)]
mod file_updater_test;

use crate::app_error::AppError;
use crate::cli::CliArgs;
use crate::config::Config;
use crate::llm;
use crate::logger;
use crate::system_prompts::{
    CODE_MODIFICATION_INSTRUCTIONS, COMMITTING_CODE_CONTEXT_QUERY, COMMITTING_CODE_INITIAL_QUERY,
    COMMITTING_CODE_REFACTOR_QUERY, COMMITTING_CODE_REPAIR_QUERY, PROJECT_STRUCTURE,
};
use file_updater as file_updater_impl;
use response_parser as response_parser_impl;
use std::collections::HashMap;
use std::path::PathBuf;

const MAX_ATTEMPTS: u32 = 4;

pub async fn run(logger: &logger::Logger, cli_args: CliArgs) -> Result<(), AppError> {
    let config = Config::load(&cli_args)?;

    println!("Building codebase context for LLM...");
    let codebase = build_codebase(&config, logger).await?;
    logger.log_text("codebase.txt", &codebase)?;

    let mut last_build_output: Option<String> = None;
    let mut cumulative_updates: HashMap<PathBuf, Option<String>> = HashMap::new();

    for attempt in 1..=MAX_ATTEMPTS {
        println!("Starting attempt {attempt}/{MAX_ATTEMPTS}...");

        let (prompt, name_part) = if attempt == 1 {
            (
                build_initial_prompt(&config, &cli_args, &codebase),
                "initial-query",
            )
        } else {
            let build_output = last_build_output
                .as_ref()
                .expect("Build output should exist for repair attempts");
            (
                build_repair_prompt(&config, build_output, &cumulative_updates, &codebase),
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
        let updates = response_parser_impl::parse_llm_response(&response_text)?;

        for update in &updates {
            cumulative_updates.insert(update.path.clone(), update.content.clone());
        }

        file_updater_impl::apply_updates(&updates)?;

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

async fn build_codebase(config: &Config, logger: &logger::Logger) -> Result<String, AppError> {
    let codebase_summary = context_builder::build_codebase_summary()?;

    let prompt = format!(
        "{}\n{}\n[user query]\n{}\n[codebase summary]\n{}",
        PROJECT_STRUCTURE, COMMITTING_CODE_CONTEXT_QUERY, config.query, codebase_summary
    );

    let response_text = llm::query(
        config.model,
        config.api_key.clone(),
        &prompt,
        logger,
        "0-context-query",
    )
    .await?;

    let file_paths = response_parser_impl::parse_context_llm_response(&response_text)?;

    let protection = file_updater_impl::PathProtection::new()?;
    let mut codebase = String::new();
    for path in file_paths {
        protection.validate(&path)?;
        let content = std::fs::read_to_string(&path).map_err(|e| {
            AppError::FileUpdate(format!(
                "Failed to read file for codebase {}: {}",
                path.display(),
                e
            ))
        })?;
        codebase.push_str(&format!("--- {} ---\n", path.display()));
        codebase.push_str(&content);
        if !content.ends_with('\n') {
            codebase.push('\n');
        }
        codebase.push('\n');
    }

    Ok(codebase)
}

fn build_initial_prompt(config: &Config, cli_args: &CliArgs, codebase: &str) -> String {
    let initial_query_prompt = if cli_args.refactor {
        COMMITTING_CODE_REFACTOR_QUERY
    } else {
        COMMITTING_CODE_INITIAL_QUERY
    };
    let system_prompt =
        format!("{PROJECT_STRUCTURE}\n{CODE_MODIFICATION_INSTRUCTIONS}\n{initial_query_prompt}");
    format!(
        "{}\n[query]\n{}\n[codebase]\n{}",
        system_prompt, config.query, codebase
    )
}

fn build_repair_prompt(
    config: &Config,
    build_output: &str,
    file_replacements: &HashMap<PathBuf, Option<String>>,
    codebase: &str,
) -> String {
    let replacements_str = format_file_replacements(file_replacements);
    let system_prompt = format!(
        "{PROJECT_STRUCTURE}\n{CODE_MODIFICATION_INSTRUCTIONS}\n{COMMITTING_CODE_REPAIR_QUERY}"
    );
    format!(
        "{}\n[build.sh output]\n{}\n[query]\n{}\n[codebase]\n{}\n[file replacements]\n{}",
        system_prompt, build_output, config.query, codebase, replacements_str
    )
}

fn format_file_replacements(replacements: &HashMap<PathBuf, Option<String>>) -> String {
    let mut result = String::new();
    let mut sorted_replacements: Vec<_> = replacements.iter().collect();
    sorted_replacements.sort_by_key(|(path, _)| (*path).clone());

    for (path, content_opt) in sorted_replacements {
        let path_str = path.to_string_lossy();
        match content_opt {
            Some(content) => {
                result.push_str(&format!("--- FILE REPLACEMENT {path_str} ---\n"));
                result.push_str(content);
                if !content.ends_with('\n') {
                    result.push('\n');
                }
                result.push('\n');
            }
            None => {
                result.push_str(&format!("--- FILE REMOVED {path_str} ---\n\n"));
            }
        }
    }
    result
}
