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

fn set_progress(root: &Path, rel_path: &str, stage: Stage, content: &str) {
    let state_dir = root.join("agent-state/specifications").join(rel_path);
    fs::create_dir_all(&state_dir).unwrap();
    fs::write(state_dir.join(stage.as_str()), content).unwrap();
}

#[test]
fn test_find_next_task_alphabetical() {
    let temp = setup_project();
    let root = temp.path();

    // Create 3 specs: B, A, C. All at progress 0.
    create_spec(root, "B");
    create_spec(root, "A");
    create_spec(root, "C");

    // find_next_task should return A
    let task = find_next_task(root).unwrap().expect("Should find a task");
    assert!(
        task.spec_path.ends_with("A/UserSpecification.md"),
        "Expected A, got {:?}",
        task.spec_path
    );
}

#[test]
fn test_find_next_task_progress_priority() {
    let temp = setup_project();
    let root = temp.path();

    // A is progress 1 (self-consistent done)
    // B is progress 0
    create_spec(root, "A");
    create_spec(root, "B");

    set_progress(root, "A", Stage::SelfConsistent, "# Spec");

    // Should pick B because it has lower progress (0 vs 1)
    let task = find_next_task(root).unwrap().expect("Should find a task");
    assert!(
        task.spec_path.ends_with("B/UserSpecification.md"),
        "Expected B, got {:?}",
        task.spec_path
    );
}

#[test]
fn test_find_next_task_same_progress_alphabetical_deep() {
    let temp = setup_project();
    let root = temp.path();

    // src/llm/UserSpecification.md
    // src/cli/UserSpecification.md
    // Both progress 0. 'src/cli' comes before 'src/llm'.
    create_spec(root, "src/llm");
    create_spec(root, "src/cli");

    let task = find_next_task(root).unwrap().expect("Should find a task");
    // Depending on path separator, but 'c' < 'l'.
    assert!(
        task.spec_path.ends_with("src/cli/UserSpecification.md"),
        "Expected src/cli, got {:?}",
        task.spec_path
    );
}

#[test]
fn test_find_next_task_cache_invalidation() {
    let temp = setup_project();
    let root = temp.path();

    create_spec(root, "A");
    // Set progress for stage 1, but with different content
    set_progress(root, "A", Stage::SelfConsistent, "Old Content");

    // Because the content doesn't match "# Spec", progress should be 0, next stage SelfConsistent
    let task = find_next_task(root).unwrap().expect("Should find a task");
    assert_eq!(task.stage, Stage::SelfConsistent);
}

#[test]
fn test_find_next_task_cache_match() {
    let temp = setup_project();
    let root = temp.path();

    create_spec(root, "A");
    // Set progress for stage 1, with matching content
    set_progress(root, "A", Stage::SelfConsistent, "# Spec");

    // Progress should be 1, next stage ProjectConsistent
    let task = find_next_task(root).unwrap().expect("Should find a task");
    assert_eq!(task.stage, Stage::ProjectConsistent);
}
