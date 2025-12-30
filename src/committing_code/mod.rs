mod build_runner;
mod file_updater;
mod git_status;
mod response_parser;

#[cfg(test)]
mod file_updater_gitignore_tests;
#[cfg(test)]
mod file_updater_test;
#[cfg(test)]
mod git_status_test;
#[cfg(test)]
mod response_parser_adversarial_test;
#[cfg(test)]
mod response_parser_edge_test;
#[cfg(test)]
mod response_parser_error_test;
#[cfg(test)]
mod response_parser_extra_test;
#[cfg(test)]
mod response_parser_happy_test;

use crate::app_error::AppError;
use crate::cli::CliArgs;
use crate::config::Config;
use crate::context_builder;
use crate::llm;
use crate::logger;
use crate::system_prompts::{
    CODE_MODIFICATION_INSTRUCTIONS, COMMITTING_CODE_EXTRA_CODE_QUERY,
    COMMITTING_CODE_INITIAL_QUERY, COMMITTING_CODE_REPAIR_QUERY, PROJECT_STRUCTURE,
};
use file_updater as file_updater_impl;
use git_status as git_status_impl;
use response_parser as response_parser_impl;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

const MAX_ATTEMPTS: u32 = 4;

pub async fn run(logger: &logger::Logger, cli_args: CliArgs) -> Result<(), AppError> {
    // Ensure .gitignore protects agent-config before proceeding
    git_status_impl::verify_gitignore_protection()?;

    if !cli_args.force {
        println!("Checking for uncommitted changes...");
        git_status_impl::check_for_uncommitted_changes()?;
    }

    let config = Config::load(&cli_args)?;

    println!("Building codebase context for LLM...");
    let initial_query_prompt = COMMITTING_CODE_INITIAL_QUERY;
    let system_prompt_part =
        format!("{PROJECT_STRUCTURE}\n{CODE_MODIFICATION_INSTRUCTIONS}\n{initial_query_prompt}");
    let next_agent_prompt = format!(
        "{}\n[supervisor query]\n{}",
        system_prompt_part, config.query
    );

    let codebase = context_builder::build_codebase_context(
        &next_agent_prompt,
        &config,
        logger,
        "0-context-query",
    )
    .await?;
    logger.log_text("codebase.txt", &codebase)?;

    let _ = run_with_codebase(logger, &config, codebase).await?;

    Ok(())
}

pub async fn run_with_codebase(
    logger: &logger::Logger,
    config: &Config,
    mut codebase: String,
) -> Result<String, AppError> {
    let initial_query_prompt = COMMITTING_CODE_INITIAL_QUERY;
    let system_prompt_part =
        format!("{PROJECT_STRUCTURE}\n{CODE_MODIFICATION_INSTRUCTIONS}\n{initial_query_prompt}");
    let next_agent_prompt = format!(
        "{}\n[supervisor query]\n{}",
        system_prompt_part, config.query
    );

    let mut last_build_output: Option<String> = None;
    let mut cumulative_updates: HashMap<PathBuf, Option<String>> = HashMap::new();

    for attempt in 1..=MAX_ATTEMPTS {
        println!("Starting attempt {attempt}/{MAX_ATTEMPTS}...");

        let (prompt, name_part) = if attempt == 1 {
            (
                build_initial_prompt(&next_agent_prompt, &codebase),
                "initial-query",
            )
        } else {
            let build_output = last_build_output
                .as_ref()
                .expect("Build output should exist for repair attempts");
            (
                build_repair_prompt(config, build_output, &cumulative_updates, &codebase),
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
                return Ok(response_text);
            }
            Err(build_failure) => {
                logger.log_text(&format!("{log_prefix}-build.txt"), &build_failure.output)?;
                println!("Build failed. Preparing for repair attempt...");
                last_build_output = Some(build_failure.output);

                if attempt < MAX_ATTEMPTS {
                    let build_output = last_build_output.as_ref().unwrap();
                    run_extra_code_query(config, logger, build_output, &mut codebase, attempt)
                        .await?;
                }
            }
        }
    }

    println!("Build did not pass after {MAX_ATTEMPTS} attempts. Aborting.");
    Err(AppError::MaxAttemptsReached)
}

async fn run_extra_code_query(
    config: &Config,
    logger: &logger::Logger,
    build_output: &str,
    codebase: &mut String,
    attempt: u32,
) -> Result<(), AppError> {
    println!("Running extra code query to check for missing context...");
    let existing_files = extract_filenames_from_codebase(codebase);
    let existing_files_list = existing_files.join("\n");

    let extra_code_prompt = format!(
        "{COMMITTING_CODE_EXTRA_CODE_QUERY}\n[codebase file list]\n{existing_files_list}\n[build.sh output]\n{build_output}"
    );

    let response = llm::query(
        config.model,
        config.api_key.clone(),
        &extra_code_prompt,
        logger,
        &format!("{attempt}-extra-code"),
    )
    .await?;

    let extra_files = response_parser_impl::parse_extra_files_response(&response)?;
    let mut files_added = 0;

    let protection = file_updater_impl::PathProtection::new()?;

    for path in extra_files {
        let path_str = path.to_string_lossy().to_string();
        if existing_files.contains(&path_str) {
            continue;
        }

        // Validate the path before reading to prevent leaking protected files
        if protection.validate(&path).is_err() {
            println!("Skipping protected or ignored file: {path_str}");
            continue;
        }

        if let Ok(content) = fs::read_to_string(&path) {
            codebase.push_str(&format!("--- {path_str} ---\n"));
            codebase.push_str(&content);
            if !content.ends_with('\n') {
                codebase.push('\n');
            }
            codebase.push('\n');
            files_added += 1;
        }
    }

    if files_added > 0 {
        println!("Added {files_added} extra files to context.");
    }

    Ok(())
}

fn extract_filenames_from_codebase(codebase: &str) -> Vec<String> {
    codebase
        .lines()
        .filter_map(|line| {
            if line.starts_with("--- ") && line.ends_with(" ---") {
                let name = line.trim_start_matches("--- ").trim_end_matches(" ---");
                if name == "FILENAMES"
                    || name == "END FILENAMES"
                    || name.starts_with("FILE REPLACEMENT")
                    || name.starts_with("FILE REMOVED")
                {
                    None
                } else {
                    Some(name.to_string())
                }
            } else {
                None
            }
        })
        .collect()
}

fn build_initial_prompt(next_agent_prompt: &str, codebase: &str) -> String {
    format!("{next_agent_prompt}\n[codebase]\n{codebase}")
}

fn build_repair_prompt(
    config: &Config,
    build_output: &str,
    file_replacements: &std::collections::HashMap<std::path::PathBuf, Option<String>>,
    codebase: &str,
) -> String {
    let replacements_str = format_file_replacements(file_replacements);
    let system_prompt = format!(
        "{PROJECT_STRUCTURE}\n{CODE_MODIFICATION_INSTRUCTIONS}\n{COMMITTING_CODE_REPAIR_QUERY}"
    );
    format!(
        "{}\n[build.sh output]\n{}\n[supervisor query]\n{}\n[codebase]\n{}\n[file replacements]\n{}",
        system_prompt, build_output, config.query, codebase, replacements_str
    )
}

fn format_file_replacements(
    replacements: &std::collections::HashMap<std::path::PathBuf, Option<String>>,
) -> String {
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
