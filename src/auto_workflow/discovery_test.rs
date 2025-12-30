use super::discovery::find_next_task;
use crate::auto_workflow::types::Stage;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn setup_project() -> TempDir {
    let dir = tempfile::Builder::new()
        .prefix("test-project")
        .tempdir()
        .unwrap();

    // Create .gitignore
    let gitignore = dir.path().join(".gitignore");
    fs::write(gitignore, "/agent-config\n").unwrap();

    dir
}

fn create_spec(root: &Path, rel_path: &str) {
    let path = root.join(rel_path).join("UserSpecification.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, "# Spec").unwrap();
}

fn create_module_dep(root: &Path, module: &str, dep: &str) {
    let path = root.join(module).join("ModuleDependencies.md");
    fs::write(path, format!("# Deps\n\n{dep}")).unwrap();
}

fn set_progress(root: &Path, rel_path: &str, stage: Stage, content: &str) {
    let state_dir = root.join("agent-state/specifications").join(rel_path);
    fs::create_dir_all(&state_dir).unwrap();
    fs::write(state_dir.join(stage.as_str()), content).unwrap();
}

#[test]
fn test_find_next_task_stage_priority() {
    let temp = setup_project();
    let root = temp.path();

    create_spec(root, "A");
    // A has passed SelfConsistent
    set_progress(root, "A", Stage::SelfConsistent, "# Spec");

    create_spec(root, "B");
    // B has NOT passed SelfConsistent

    // Expect B (SelfConsistent) over A (Implemented)
    let task = find_next_task(root).unwrap().expect("Should find task");
    assert!(task.spec_path.ends_with("B/UserSpecification.md"));
    assert_eq!(task.stage, Stage::SelfConsistent);
}

#[test]
fn test_find_next_task_level_priority() {
    let temp = setup_project();
    let root = temp.path();

    // A depends on B
    create_spec(root, "src/A");
    create_spec(root, "src/B");
    create_module_dep(root, "src/A", "src/B");

    // Both at SelfConsistent stage
    // B is Level 0 (no deps). A is Level 1.
    // Expect B first.

    let task = find_next_task(root).unwrap().expect("Should find task");
    assert!(task.spec_path.ends_with("src/B/UserSpecification.md"));
}
