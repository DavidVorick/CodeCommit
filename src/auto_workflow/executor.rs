use crate::app_error::AppError;
use crate::auto_workflow::prompts::{
    COMPLETE, PROJECT_CONSISTENT, RESPONSE_FORMAT_INSTRUCTIONS, SECURE, SELF_CONSISTENT,
};
use crate::auto_workflow::types::Stage;
use crate::config::Config;
use crate::llm;
use crate::logger::Logger;
use ignore::WalkBuilder;
use std::fs;
use std::path::Path;

pub enum ExecutionResult {
    Success,
    ChangesRequested,
    Failure,
}

pub async fn execute_task(
    task_spec_path: &Path,
    stage: Stage,
    config: &Config,
    logger: &Logger,
) -> Result<ExecutionResult, AppError> {
    println!(
        "Executing Auto Workflow: {} for {}",
        stage,
        task_spec_path.display()
    );

    let spec_content = fs::read_to_string(task_spec_path).map_err(|e| {
        AppError::FileUpdate(format!(
            "Failed to read spec {}: {}",
            task_spec_path.display(),
            e
        ))
    })?;

    let prompt = match stage {
        Stage::SelfConsistent => build_self_consistent_prompt(&spec_content),
        Stage::ProjectConsistent => build_project_consistent_prompt(task_spec_path, &spec_content)?,
        Stage::Complete => build_complete_prompt(task_spec_path, &spec_content)?,
        Stage::Secure => build_secure_prompt(task_spec_path, &spec_content)?,
    };

    let response = llm::query(
        config.model,
        config.api_key.clone(),
        &prompt,
        logger,
        &format!("auto-workflow-{stage}"),
    )
    .await?;

    if response.contains("@@@@task-success@@@@") {
        mark_stage_complete(task_spec_path, stage, &spec_content)?;
        Ok(ExecutionResult::Success)
    } else if response.contains("@@@@changes-requested@@@@") {
        extract_and_print_comment(&response);
        Ok(ExecutionResult::ChangesRequested)
    } else if response.contains("@@@@changes-attempted@@@@") {
        // Not expected for these stages, but treat as failure/stop
        println!("Unexpected status 'changes-attempted' for review stage.");
        Ok(ExecutionResult::Failure)
    } else {
        println!("LLM response did not contain a valid status code.");
        Ok(ExecutionResult::Failure)
    }
}

fn build_self_consistent_prompt(spec_content: &str) -> String {
    format!(
        "{}\n[response format instructions]\n{}\n[self consistent prompt]\n{}\n[target user specification]\n{}",
        "1. self-consistent",
        RESPONSE_FORMAT_INSTRUCTIONS,
        SELF_CONSISTENT,
        spec_content
    )
}

fn build_project_consistent_prompt(
    spec_path: &Path,
    spec_content: &str,
) -> Result<String, AppError> {
    let parent_spec_content = find_parent_spec(spec_path)?;
    let child_specs_content = find_child_specs(spec_path)?;

    Ok(format!(
        "{}\n[response format instructions]\n{}\n[project-consistent prompt]\n{}\n[target user specification]\n{}\n[parent user specification]\n{}\n[all child user specifications]\n{}",
        "2. project-consistent",
        RESPONSE_FORMAT_INSTRUCTIONS,
        PROJECT_CONSISTENT,
        spec_content,
        parent_spec_content,
        child_specs_content
    ))
}

fn build_complete_prompt(spec_path: &Path, spec_content: &str) -> Result<String, AppError> {
    let parent_spec_content = find_parent_spec(spec_path)?;
    let child_specs_content = find_child_specs(spec_path)?;

    Ok(format!(
        "{}\n[response format instructions]\n{}\n[complete prompt]\n{}\n[target user specification]\n{}\n[parent user specification]\n{}\n[all child user specifications]\n{}",
        "3. complete",
        RESPONSE_FORMAT_INSTRUCTIONS,
        COMPLETE,
        spec_content,
        parent_spec_content,
        child_specs_content
    ))
}

fn build_secure_prompt(spec_path: &Path, spec_content: &str) -> Result<String, AppError> {
    let parent_spec_content = find_parent_spec(spec_path)?;
    let child_specs_content = find_child_specs(spec_path)?;

    Ok(format!(
        "{}\n[response format instructions]\n{}\n[secure prompt]\n{}\n[target user specification]\n{}\n[parent user specification]\n{}\n[all child user specifications]\n{}",
        "4. secure",
        RESPONSE_FORMAT_INSTRUCTIONS,
        SECURE,
        spec_content,
        parent_spec_content,
        child_specs_content
    ))
}

fn find_parent_spec(spec_path: &Path) -> Result<String, AppError> {
    // Look in parent directories for UserSpecification.md
    let mut current = spec_path.parent().unwrap_or(Path::new("."));
    // If we are at spec_path, parent dir is current.parent
    if current == Path::new("") {
        current = Path::new(".");
    }

    // We start searching from the parent of the directory containing the spec
    // If spec is src/llm/UserSpecification.md, dir is src/llm. Parent search starts at src.
    let mut search_dir = current.parent();

    while let Some(dir) = search_dir {
        let candidate = dir.join("UserSpecification.md");
        if candidate.exists() {
            return fs::read_to_string(&candidate)
                .map_err(|e| AppError::FileUpdate(format!("Failed to read parent spec: {e}")));
        }
        if dir == Path::new(".") || dir == Path::new("") {
            break;
        }
        search_dir = dir.parent();
    }

    Ok("No parent specification found.".to_string())
}

fn find_child_specs(spec_path: &Path) -> Result<String, AppError> {
    let base_dir = spec_path.parent().unwrap_or(Path::new("."));
    let mut content = String::new();

    // Walk subdirectories looking for UserSpecification.md
    // We must exclude the current spec itself.
    let walker = WalkBuilder::new(base_dir)
        .hidden(false)
        .git_ignore(true)
        .build();

    for result in walker {
        let entry = result.map_err(|e| AppError::Config(format!("Failed to walk dir: {e}")))?;
        if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false)
            && entry.file_name() == "UserSpecification.md"
        {
            let path = entry.path();
            if path != spec_path {
                let text = fs::read_to_string(path)
                    .map_err(|e| AppError::FileUpdate(format!("Failed to read child spec: {e}")))?;
                content.push_str(&format!(
                    "\n--- Child Spec: {} ---\n{}\n",
                    path.display(),
                    text
                ));
            }
        }
    }

    if content.is_empty() {
        Ok("No child specifications found.".to_string())
    } else {
        Ok(content)
    }
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
    if let Some(start) = response.find("%%%%comment%%%%") {
        if let Some(end) = response.find("%%%%end%%%%") {
            let comment = &response[start + 15..end];
            println!("\n[Auto Workflow Comment]\n{}", comment.trim());
        }
    }
}
