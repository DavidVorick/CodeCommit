use crate::app_error::AppError;
use crate::auto_workflow::prompts::{
    COMPLETE, PROJECT_CONSISTENT, RESPONSE_FORMAT_INSTRUCTIONS, SECURE, SELF_CONSISTENT,
};
use crate::auto_workflow::types::Stage;
use crate::config::Config;
use crate::llm;
use crate::logger::Logger;
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
        Stage::ProjectConsistent => build_project_consistent_prompt(&spec_content)?,
        Stage::Complete => build_complete_prompt(&spec_content)?,
        Stage::Secure => build_secure_prompt(&spec_content)?,
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

fn build_project_consistent_prompt(spec_content: &str) -> Result<String, AppError> {
    let root = Path::new(".");
    let dependency_specs = find_dependency_specs(root, spec_content)?;

    Ok(format!(
        "{}\n[response format instructions]\n{}\n[project-consistent prompt]\n{}\n[target user specification]\n{}\n[all dependency user specifications]\n{}",
        "2. project-consistent",
        RESPONSE_FORMAT_INSTRUCTIONS,
        PROJECT_CONSISTENT,
        spec_content,
        dependency_specs
    ))
}

fn build_complete_prompt(spec_content: &str) -> Result<String, AppError> {
    let root = Path::new(".");
    let dependency_specs = find_dependency_specs(root, spec_content)?;
    let full_module_list = get_full_module_list(root)?;

    Ok(format!(
        "{}\n[response format instructions]\n{}\n[complete prompt]\n{}\n[target user specification]\n{}\n[all dependency user specifications]\n{}\n[full list of modules]\n{}",
        "3. complete",
        RESPONSE_FORMAT_INSTRUCTIONS,
        COMPLETE,
        spec_content,
        dependency_specs,
        full_module_list
    ))
}

fn build_secure_prompt(spec_content: &str) -> Result<String, AppError> {
    let root = Path::new(".");
    let dependency_specs = find_dependency_specs(root, spec_content)?;

    Ok(format!(
        "{}\n[response format instructions]\n{}\n[secure prompt]\n{}\n[target user specification]\n{}\n[all dependency user specifications]\n{}",
        "4. secure",
        RESPONSE_FORMAT_INSTRUCTIONS,
        SECURE,
        spec_content,
        dependency_specs
    ))
}

fn find_dependency_specs(root: &Path, spec_content: &str) -> Result<String, AppError> {
    let mut dependencies = Vec::new();
    let mut in_deps = false;
    let mut in_code_block = false;

    for line in spec_content.lines() {
        if line.trim().starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }
        if in_code_block {
            continue;
        }

        let trimmed = line.trim();
        if trimmed == "dependencies:" {
            in_deps = true;
            continue;
        }
        if in_deps {
            if trimmed.is_empty() {
                continue;
            }
            if !line.starts_with(' ') && !line.starts_with('\t') {
                break;
            }
            dependencies.push(trimmed.to_string());
        }
    }

    if dependencies.is_empty() {
        return Ok("No dependencies declared.".to_string());
    }

    let mut content = String::new();
    for dep in dependencies {
        let dep_path = root.join(&dep).join("UserSpecification.md");
        if dep_path.exists() {
            let text = fs::read_to_string(&dep_path).map_err(|e| {
                AppError::FileUpdate(format!(
                    "Failed to read dependency spec {}: {}",
                    dep_path.display(),
                    e
                ))
            })?;
            content.push_str(&format!("\n--- Dependency Spec: {dep} ---\n{text}\n"));
        } else {
            content.push_str(&format!(
                "\n--- Dependency Spec: {dep} ---\n[File Not Found]\n"
            ));
        }
    }

    Ok(content)
}

fn get_full_module_list(root: &Path) -> Result<String, AppError> {
    let specs = crate::auto_workflow::discovery::find_all_user_specifications(root)?;
    let mut modules = Vec::new();
    for spec in specs {
        let module_dir = spec.parent().unwrap_or(root);
        let relative = module_dir.strip_prefix(root).unwrap_or(module_dir);
        modules.push(relative.display().to_string());
    }
    modules.sort();
    Ok(modules.join("\n"))
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
