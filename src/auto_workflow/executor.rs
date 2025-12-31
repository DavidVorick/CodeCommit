use crate::app_error::AppError;
use crate::auto_workflow::file_updater;
use crate::auto_workflow::prompt_builder;
use crate::auto_workflow::types::{Stage, Task};
use crate::committing_code;
use crate::config::Config;
use crate::llm;
use crate::logger::Logger;
use crate::system_prompts;
use std::fs;
use std::path::Path;

pub enum ExecutionResult {
    Success,
    ChangesRequested,
    ChangesAttempted,
    Failure,
}

pub async fn execute_task(
    task: &Task,
    config: &Config,
    logger: &Logger,
) -> Result<ExecutionResult, AppError> {
    println!(
        "Executing Auto Workflow: {} for {}",
        task.stage,
        task.spec_path.display()
    );

    let spec_content = fs::read_to_string(&task.spec_path).map_err(|e| {
        AppError::FileUpdate(format!(
            "Failed to read spec {}: {}",
            task.spec_path.display(),
            e
        ))
    })?;

    // We assume current working directory is the root for execution
    let root = Path::new(".");
    let prompt = prompt_builder::build_prompt(root, &task.spec_path, task.stage, &spec_content)?;

    // To ensure unique logs for re-runs, we might want to append a timestamp or counter,
    // but the logger handles file naming collisions or we accept overwrite.
    // The spec says: "auto-workflow-[spec-path]-[stage]"
    let log_name = format!(
        "auto-workflow-{}-{}",
        task.spec_path.display().to_string().replace('/', "+"),
        task.stage.as_str()
    );

    let response = if task.stage == Stage::SelfConsistent {
        llm::query(
            config.model,
            config.api_key.clone(),
            &prompt,
            logger,
            &log_name,
        )
        .await?
    } else {
        let task_config = Config {
            model: config.model,
            api_key: config.api_key.clone(),
            query: prompt,
            system_prompts: system_prompts::COMMITTING_CODE_INITIAL_QUERY.to_string(),
        };

        committing_code::run_with_codebase(logger, &task_config, String::new()).await?
    };

    if file_updater::has_pending_updates(&response)
        && !response.contains("@@@@changes-attempted@@@@")
    {
        return Err(AppError::FileUpdate(
            "Auto Workflow Error: Code modifications provided without 'changes-attempted' status."
                .to_string(),
        ));
    }

    validate_response_format(&response)?;

    if response.contains("@@@@task-success@@@@") {
        extract_and_print_comment(&response);
        mark_stage_complete(&task.spec_path, task.stage, &spec_content)?;
        Ok(ExecutionResult::Success)
    } else if response.contains("@@@@changes-requested@@@@") {
        extract_and_print_comment(&response);
        Ok(ExecutionResult::ChangesRequested)
    } else if response.contains("@@@@changes-attempted@@@@") {
        extract_and_print_comment(&response);
        file_updater::apply_file_updates(&response)?;
        Ok(ExecutionResult::ChangesAttempted)
    } else {
        println!("LLM response did not contain a valid status code.");
        let _ = logger.log_text(&format!("{log_name}_error.txt"), &response);
        Ok(ExecutionResult::Failure)
    }
}

pub(crate) fn validate_response_format(response: &str) -> Result<(), AppError> {
    let success = response.contains("@@@@task-success@@@@");
    let requested = response.contains("@@@@changes-requested@@@@");
    let attempted = response.contains("@@@@changes-attempted@@@@");

    let count = (success as u8) + (requested as u8) + (attempted as u8);
    if count == 0 {
        return Err(AppError::FileUpdate(
            "Auto Workflow Error: No status tag found in response.".to_string(),
        ));
    }
    if count > 1 {
        return Err(AppError::FileUpdate(
            "Auto Workflow Error: Multiple status tags found in response.".to_string(),
        ));
    }

    let comment_start_count = response.matches("%%%%comment%%%%").count();
    let comment_end_count = response.matches("%%%%end%%%%").count();

    if comment_start_count > 1 || comment_end_count > 1 {
        return Err(AppError::FileUpdate(
            "Auto Workflow Error: Multiple comment sections found in response.".to_string(),
        ));
    }

    if comment_start_count != comment_end_count {
        return Err(AppError::FileUpdate(
            "Auto Workflow Error: Mismatched comment tags.".to_string(),
        ));
    }

    Ok(())
}

fn mark_stage_complete(spec_path: &Path, stage: Stage, content: &str) -> Result<(), AppError> {
    let root = Path::new(".");
    let module_dir = spec_path.parent().unwrap_or(root);
    let relative_module_dir = module_dir.strip_prefix(root).unwrap_or(module_dir);

    let state_dir = root
        .join("agent-state")
        .join("specifications")
        .join(relative_module_dir);

    fs::create_dir_all(&state_dir).map_err(|e| {
        AppError::FileUpdate(format!(
            "Failed to create state dir {}: {}",
            state_dir.display(),
            e
        ))
    })?;

    let state_file = state_dir.join(stage.as_str());
    fs::write(&state_file, content).map_err(|e| {
        AppError::FileUpdate(format!(
            "Failed to write state file {}: {}",
            state_file.display(),
            e
        ))
    })?;

    println!("Marked {} as complete for {}", stage, spec_path.display());
    Ok(())
}

fn extract_and_print_comment(response: &str) {
    if let Some(comment) = extract_comment(response) {
        println!("\n[Auto Workflow Comment]\n{}", comment.trim());
    }
}

pub(crate) fn extract_comment(response: &str) -> Option<&str> {
    if let Some(start) = response.find("%%%%comment%%%%") {
        if let Some(end) = response.find("%%%%end%%%%") {
            return Some(&response[start + 15..end]);
        }
    }
    None
}
