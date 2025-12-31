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

#[test]
fn test_build_prompt_with_cache() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let mod_dir = root.join("src/cached_mod");
    fs::create_dir_all(&mod_dir).unwrap();
    let spec_path = mod_dir.join("UserSpecification.md");
    let spec_content = "NEW_SPEC";
    let cached_content = "OLD_SPEC";

    // Set up cache
    let cache_dir = root.join("agent-state/specifications/src/cached_mod");
    fs::create_dir_all(&cache_dir).unwrap();
    fs::write(cache_dir.join("implemented"), cached_content).unwrap();

    // Write current files
    fs::write(&spec_path, spec_content).unwrap();
    fs::write(mod_dir.join("ModuleDependencies.md"), "# Deps").unwrap();
    fs::write(root.join("UserSpecification.md"), "TOP").unwrap();

    let prompt =
        prompt_builder::build_prompt(root, &spec_path, Stage::Implemented, spec_content).unwrap();

    assert!(prompt.contains("implementation-with-cache prompt"));
    assert!(prompt.contains("cached target user specification"));
    assert!(prompt.contains(cached_content));
    assert!(prompt.contains(spec_content));
}

#[test]
fn test_build_prompt_root_module() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();

    // Root files
    fs::write(root.join("UserSpecification.md"), "TOP_LEVEL_SPEC").unwrap();
    fs::write(root.join("Cargo.toml"), "CARGO_TOML").unwrap();
    fs::write(root.join("build.sh"), "BUILD_SCRIPT").unwrap();

    // src files
    let src_dir = root.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(src_dir.join("main.rs"), "fn main() {}").unwrap();

    // Submodule (should not be included in root context)
    let sub_dir = src_dir.join("sub");
    fs::create_dir_all(&sub_dir).unwrap();
    fs::write(sub_dir.join("lib.rs"), "fn sub() {}").unwrap();

    let spec_path = root.join("UserSpecification.md");
    let prompt =
        prompt_builder::build_prompt(root, &spec_path, Stage::Implemented, "TOP_LEVEL_SPEC")
            .unwrap();

    // Should include root files
    assert!(prompt.contains("TOP_LEVEL_SPEC"));
    assert!(prompt.contains("CARGO_TOML"));
    assert!(prompt.contains("BUILD_SCRIPT"));

    // Should include src/main.rs
    assert!(prompt.contains("fn main() {}"));

    // Should NOT include submodule files (unless deps, but here no deps)
    assert!(!prompt.contains("fn sub() {}"));
}
