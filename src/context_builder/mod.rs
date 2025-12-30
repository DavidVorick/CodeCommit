mod path_filter;
mod response_parser;
mod summary_builder;

#[cfg(test)]
mod context_assembly_test;
#[cfg(test)]
mod path_filter_test;
#[cfg(test)]
mod response_parser_test_errors;
#[cfg(test)]
mod response_parser_test_happy;
#[cfg(test)]
mod summary_builder_test;

use crate::app_error::AppError;
use crate::config::Config;
use crate::llm;
use crate::logger::Logger;
use crate::system_prompts::CONTEXT_BUILDER_CONTEXT_QUERY;
use std::path::{Path, PathBuf};

pub async fn build_codebase_context(
    next_agent_full_prompt: &str,
    config: &Config,
    logger: &Logger,
    log_prefix: &str,
) -> Result<String, AppError> {
    let codebase_summary = summary_builder::build_summary()?;

    let prompt = format!(
        "{CONTEXT_BUILDER_CONTEXT_QUERY}\n\n=== Next Agent Full Prompt ===\n{next_agent_full_prompt}\n\n=== Codebase Summary ===\n{codebase_summary}"
    );

    let response_text = llm::query(
        config.model,
        config.api_key.clone(),
        &prompt,
        logger,
        log_prefix,
    )
    .await?;

    let file_paths = response_parser::parse_context_llm_response(&response_text)?;

    load_context_files(file_paths)
}

fn load_context_files(file_paths: Vec<PathBuf>) -> Result<String, AppError> {
    load_files_with_root(file_paths, Path::new("."))
}

fn load_files_with_root(file_paths: Vec<PathBuf>, root: &Path) -> Result<String, AppError> {
    let filter = path_filter::PathFilter::new_for_base_dir(root)?;
    let mut codebase = String::new();
    for path in file_paths {
        filter.validate(&path)?;
        let full_path = root.join(&path);
        let content = std::fs::read_to_string(&full_path).map_err(|e| {
            AppError::FileUpdate(format!(
                "Failed to read file for codebase {}: {}",
                full_path.display(),
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
