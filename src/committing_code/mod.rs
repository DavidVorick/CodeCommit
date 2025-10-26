use crate::app_error::AppError;
use crate::build_runner;
use crate::cli::CliArgs;
use crate::config::Config;
use crate::file_updater;
use crate::llm;
use crate::logger;
use crate::response_parser;
use crate::system_prompts::{
    CODE_MODIFICATION_INSTRUCTIONS, COMMITTING_CODE_INITIAL_QUERY, COMMITTING_CODE_REFACTOR_QUERY,
    COMMITTING_CODE_REPAIR_QUERY, PROJECT_STRUCTURE,
};
use std::collections::HashMap;
use std::path::PathBuf;

const MAX_ATTEMPTS: u32 = 4;

pub async fn run(logger: &logger::Logger, cli_args: CliArgs) -> Result<(), AppError> {
    let config = Config::load(&cli_args)?;

    let mut last_build_output: Option<String> = None;
    let mut cumulative_updates: HashMap<PathBuf, Option<String>> = HashMap::new();

    for attempt in 1..=MAX_ATTEMPTS {
        println!("Starting attempt {attempt}/{MAX_ATTEMPTS}...");

        let (prompt, name_part) = if attempt == 1 {
            (build_initial_prompt(&config, &cli_args), "initial-query")
        } else {
            let build_output = last_build_output
                .as_ref()
                .expect("Build output should exist for repair attempts");
            (
                build_repair_prompt(&config, build_output, &cumulative_updates),
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

fn build_initial_prompt(config: &Config, cli_args: &CliArgs) -> String {
    let initial_query_prompt = if cli_args.refactor {
        COMMITTING_CODE_REFACTOR_QUERY
    } else {
        COMMITTING_CODE_INITIAL_QUERY
    };
    let system_prompt =
        format!("{PROJECT_STRUCTURE}\n{CODE_MODIFICATION_INSTRUCTIONS}\n{initial_query_prompt}");
    format!(
        "{}\n[query]\n{}\n[codebase]\n{}",
        system_prompt, config.query, config.code_rollup
    )
}

fn build_repair_prompt(
    config: &Config,
    build_output: &str,
    file_replacements: &HashMap<PathBuf, Option<String>>,
) -> String {
    let replacements_str = format_file_replacements(file_replacements);
    let system_prompt = format!(
        "{PROJECT_STRUCTURE}\n{CODE_MODIFICATION_INSTRUCTIONS}\n{COMMITTING_CODE_REPAIR_QUERY}"
    );
    format!(
        "{}\n[build.sh output]\n{}\n[query]\n{}\n[codebase]\n{}\n[file replacements]\n{}",
        system_prompt, build_output, config.query, config.code_rollup, replacements_str
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
