use super::prompt_builder;
use crate::auto_workflow::types::Stage;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_build_prompt_includes_files() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();

    // Setup files
    fs::write(root.join("UserSpecification.md"), "TOP_LEVEL_SPEC").unwrap();
    fs::write(root.join("Cargo.toml"), "CARGO_TOML").unwrap();

    let mod_dir = root.join("src/testmod");
    fs::create_dir_all(&mod_dir).unwrap();
    let spec_path = mod_dir.join("UserSpecification.md");
    fs::write(&spec_path, "MODULE_SPEC").unwrap();
    fs::write(mod_dir.join("lib.rs"), "fn test() {}").unwrap();
    fs::write(mod_dir.join("ModuleDependencies.md"), "# Deps").unwrap();

    let prompt =
        prompt_builder::build_prompt(root, &spec_path, Stage::Implemented, "MODULE_SPEC").unwrap();

    assert!(prompt.contains("TOP_LEVEL_SPEC"));
    assert!(prompt.contains("CARGO_TOML"));
    assert!(prompt.contains("MODULE_SPEC"));
    assert!(prompt.contains("fn test() {}"));
}

#[test]
fn test_build_prompt_includes_dependencies() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();

    fs::write(root.join("UserSpecification.md"), "TOP").unwrap();

    let dep_dir = root.join("src/dep");
    fs::create_dir_all(&dep_dir).unwrap();
    fs::write(dep_dir.join("UserSpecification.md"), "DEP_SPEC").unwrap();
    fs::write(dep_dir.join("APISignatures.md"), "DEP_SIG").unwrap();

    let mod_dir = root.join("src/mainmod");
    fs::create_dir_all(&mod_dir).unwrap();
    let spec_path = mod_dir.join("UserSpecification.md");
    fs::write(&spec_path, "MAIN_SPEC").unwrap();
    fs::write(
        mod_dir.join("ModuleDependencies.md"),
        "# Module Dependencies\n\nsrc/dep",
    )
    .unwrap();

    let prompt =
        prompt_builder::build_prompt(root, &spec_path, Stage::Implemented, "MAIN_SPEC").unwrap();

    assert!(prompt.contains("DEP_SPEC"));
    assert!(prompt.contains("DEP_SIG"));
}

#[test]
fn test_build_prompt_documented_stage() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();

    let mod_dir = root.join("src/docmod");
    fs::create_dir_all(&mod_dir).unwrap();
    let spec_path = mod_dir.join("UserSpecification.md");
    fs::write(&spec_path, "DOC_SPEC").unwrap();
    fs::write(mod_dir.join("internal.rs"), "struct Internal;").unwrap();

    let prompt =
        prompt_builder::build_prompt(root, &spec_path, Stage::Documented, "DOC_SPEC").unwrap();

    // Documented stage should focus on module internals
    assert!(prompt.contains("3. documented"));
    assert!(prompt.contains("DOC_SPEC"));
    assert!(prompt.contains("struct Internal;"));
}
