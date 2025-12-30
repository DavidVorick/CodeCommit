use crate::auto_workflow::discovery::find_next_task;
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

fn create_spec(root: &Path, rel_path: &str, content: &str) {
    let path = root.join(rel_path).join("UserSpecification.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();

    // Ensure ModuleDependencies.md exists
    let dep_path = root.join(rel_path).join("ModuleDependencies.md");
    if !dep_path.exists() {
        fs::write(dep_path, "# Deps\n").unwrap();
    }
}

fn set_progress(root: &Path, rel_path: &str, stage: Stage, content: &str) {
    let state_dir = root.join("agent-state/specifications").join(rel_path);
    fs::create_dir_all(&state_dir).unwrap();
    fs::write(state_dir.join(stage.as_str()), content).unwrap();
}

#[test]
fn test_phase1_progression() {
    let temp = setup_project();
    let root = temp.path();

    let module = "src/test_module";
    let spec_content = "# Test Module Spec";
    create_spec(root, module, spec_content);

    // 1. Check SelfConsistent
    let task = find_next_task(root).unwrap().expect("Should have task");
    assert_eq!(task.stage, Stage::SelfConsistent);
    assert!(task
        .spec_path
        .ends_with("src/test_module/UserSpecification.md"));

    set_progress(root, module, Stage::SelfConsistent, spec_content);

    // 2. Check Implemented
    let task = find_next_task(root).unwrap().expect("Should have task");
    assert_eq!(task.stage, Stage::Implemented);

    set_progress(root, module, Stage::Implemented, spec_content);

    // 3. Check Documented
    let task = find_next_task(root).unwrap().expect("Should have task");
    assert_eq!(task.stage, Stage::Documented);

    set_progress(root, module, Stage::Documented, spec_content);

    // 4. Check HappyPathTested
    let task = find_next_task(root).unwrap().expect("Should have task");
    assert_eq!(task.stage, Stage::HappyPathTested);

    set_progress(root, module, Stage::HappyPathTested, spec_content);

    // 5. Done
    let task = find_next_task(root).unwrap();
    assert!(task.is_none());
}

#[test]
fn test_phase1_spec_change_invalidates_steps() {
    let temp = setup_project();
    let root = temp.path();

    let module = "src/mod";
    let original_spec = "# Original";
    create_spec(root, module, original_spec);

    set_progress(root, module, Stage::SelfConsistent, original_spec);
    set_progress(root, module, Stage::Implemented, original_spec);
    set_progress(root, module, Stage::Documented, original_spec);
    set_progress(root, module, Stage::HappyPathTested, original_spec);

    // Verify done
    assert!(find_next_task(root).unwrap().is_none());

    // Update spec
    let new_spec = "# New";
    create_spec(root, module, new_spec);

    // Should revert to SelfConsistent
    let task = find_next_task(root).unwrap().expect("Should have task");
    assert_eq!(task.stage, Stage::SelfConsistent);
}
