use crate::app_error::AppError;
use crate::auto_workflow::types::Stage;
use ignore::WalkBuilder;
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

    // "one is chosen at random".
    // Since we don't have rand crate, we use simple time-based selection.
    let now = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
    let index = (now as usize) % tasks.len();
    Ok(Some(tasks.swap_remove(index)))
}

fn find_all_user_specifications(root: &Path) -> Result<Vec<PathBuf>, AppError> {
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
    // Determine progress based on existence of files in agent-state
    // Map spec_path to agent-state path.
    // Spec path: "src/llm/UserSpecification.md" -> module dir "src/llm"
    // State dir: "agent-state/specifications/src/llm"

    let module_dir = spec_path.parent().unwrap_or(root);
    let relative_module_dir = module_dir.strip_prefix(root).unwrap_or(module_dir);

    let state_base = root
        .join("agent-state")
        .join("specifications")
        .join(relative_module_dir);

    let stage1 = Stage::SelfConsistent;
    if !state_base.join(stage1.as_str()).exists() {
        return (0, Some(stage1));
    }

    let stage2 = Stage::ProjectConsistent;
    if !state_base.join(stage2.as_str()).exists() {
        return (1, Some(stage2));
    }

    // Both done (for the purpose of this implementation which only does first 2)
    (2, None)
}
