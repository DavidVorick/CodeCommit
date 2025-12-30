use crate::app_error::AppError;
use crate::auto_workflow::graph;
use crate::auto_workflow::types::{Stage, Task};
use ignore::WalkBuilder;
use std::fs;
use std::path::{Path, PathBuf};

pub fn find_next_task(root: &Path) -> Result<Option<Task>, AppError> {
    let specs = find_all_user_specifications(root)?;
    let graph_nodes = graph::build_dependency_graph(root, &specs)?;

    let mut tasks = Vec::new();

    for node in graph_nodes {
        let spec_path = node.path.join("UserSpecification.md");

        // Determine the next stage for this module
        if let Some(stage) = get_next_stage(root, &spec_path)? {
            tasks.push((stage, node.level, spec_path));
        }
    }

    if tasks.is_empty() {
        return Ok(None);
    }

    // Sort tasks:
    // 1. Level (Ascending - L0 first)
    // 2. Alphabetical (Path)
    tasks.sort_by(|(_, l1, p1), (_, l2, p2)| l1.cmp(l2).then_with(|| p1.cmp(p2)));

    let (stage, _, spec_path) = tasks.remove(0);
    Ok(Some(Task { spec_path, stage }))
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

fn get_next_stage(root: &Path, spec_path: &Path) -> Result<Option<Stage>, AppError> {
    let module_dir = spec_path.parent().unwrap_or(root);
    let relative_module_dir = module_dir.strip_prefix(root).unwrap_or(module_dir);

    let state_base = root
        .join("agent-state")
        .join("specifications")
        .join(relative_module_dir);

    let current_content = fs::read_to_string(spec_path).map_err(|_| {
        AppError::FileUpdate(format!("Could not read spec at {}", spec_path.display()))
    })?;

    // Phase 1 Steps
    if !is_stage_complete(&state_base, Stage::SelfConsistent, &current_content) {
        return Ok(Some(Stage::SelfConsistent));
    }

    if !is_stage_complete(&state_base, Stage::Implemented, &current_content) {
        return Ok(Some(Stage::Implemented));
    }

    if !is_stage_complete(&state_base, Stage::Documented, &current_content) {
        return Ok(Some(Stage::Documented));
    }

    if !is_stage_complete(&state_base, Stage::HappyPathTested, &current_content) {
        return Ok(Some(Stage::HappyPathTested));
    }

    Ok(None)
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
