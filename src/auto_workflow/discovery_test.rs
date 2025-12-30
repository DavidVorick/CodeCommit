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

    // Ensure ModuleDependencies.md exists
    let dep_path = root.join(rel_path).join("ModuleDependencies.md");
    if !dep_path.exists() {
        fs::write(dep_path, "# Deps\n").unwrap();
    }
}

fn create_module_dep(root: &Path, module: &str, dep: &str) {
    let path = root.join(module).join("ModuleDependencies.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
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

    // Both A and B are Level 0.
    // Spec requires iterating modules in order (L0 -> L1).
    // Spec also says "a module must complete all the steps of a phase before the next module is considered".
    // A comes before B alphabetically.
    // So A must complete Phase 1 before B is considered.
    // A is at Implemented. B is at SelfConsistent.
    // We expect A (Implemented).

    let task = find_next_task(root).unwrap().expect("Should find task");
    assert!(task.spec_path.ends_with("A/UserSpecification.md"));
    assert_eq!(task.stage, Stage::Implemented);
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
