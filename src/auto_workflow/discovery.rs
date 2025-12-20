use crate::app_error::AppError;
use crate::auto_workflow::types::Stage;
use ignore::WalkBuilder;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Task {
    pub spec_path: PathBuf,
    pub stage: Stage,
}

pub fn find_next_task(root: &Path) -> Result<Option<Task>, AppError> {
    let specs = find_all_user_specifications(root)?;

    let mut tasks = Vec::new();
    let mut min_progress = usize::MAX;

    for spec in specs {
        let (progress, next_stage) = get_progress_level(root, &spec);
        if progress < min_progress {
            min_progress = progress;
            tasks.clear();
        }
        if progress == min_progress {
            if let Some(stage) = next_stage {
                tasks.push(Task {
                    spec_path: spec,
                    stage,
                });
            }
        }
    }

    if tasks.is_empty() {
        return Ok(None);
    }

    // Sort tasks alphabetically by spec_path
    tasks.sort_by(|a, b| a.spec_path.cmp(&b.spec_path));

    // Return the first one
    Ok(Some(tasks.remove(0)))
}

pub(crate) fn find_all_user_specifications(root: &Path) -> Result<Vec<PathBuf>, AppError> {
    let mut specs = Vec::new();
    let walker = WalkBuilder::new(root)
        .hidden(false)
        .git_ignore(true)
        .build();

    for result in walker {
        let entry = result.map_err(|e| AppError::Config(format!("Failed to walk dir: {e}")))?;
        if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false)
            && entry.file_name() == "UserSpecification.md"
        {
            specs.push(entry.path().to_path_buf());
        }
    }
    Ok(specs)
}

fn get_progress_level(root: &Path, spec_path: &Path) -> (usize, Option<Stage>) {
    // Determine progress based on existence of files in agent-state and whether they match current spec
    // Map spec_path to agent-state path.
    // Spec path: "src/llm/UserSpecification.md" -> module dir "src/llm"
    // State dir: "agent-state/specifications/src/llm"

    let module_dir = spec_path.parent().unwrap_or(root);
    let relative_module_dir = module_dir.strip_prefix(root).unwrap_or(module_dir);

    let state_base = root
        .join("agent-state")
        .join("specifications")
        .join(relative_module_dir);

    let current_content = match fs::read_to_string(spec_path) {
        Ok(c) => c,
        Err(_) => return (0, Some(Stage::SelfConsistent)),
    };

    let stages = [
        Stage::SelfConsistent,
        Stage::ProjectConsistent,
        Stage::Complete,
        Stage::Secure,
    ];

    for (i, stage) in stages.iter().enumerate() {
        if !is_stage_complete(&state_base, *stage, &current_content) {
            return (i, Some(*stage));
        }
    }

    // All done (for the purpose of this implementation which only does first 4)
    (stages.len(), None)
}

fn is_stage_complete(state_base: &Path, stage: Stage, current_content: &str) -> bool {
    let stage_path = state_base.join(stage.as_str());
    if !stage_path.exists() {
        return false;
    }
    match fs::read_to_string(stage_path) {
        Ok(cached) => cached == current_content,
        Err(_) => false,
    }
}
