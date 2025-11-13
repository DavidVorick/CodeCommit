use crate::init::create_in_dir;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

fn exists(path: &Path) -> bool {
    path.exists()
}

fn replace_binary(template: &str, project_name: &str) -> String {
    let replaced: Vec<String> = template
        .lines()
        .map(|line| {
            let trimmed = line.trim_start();
            if trimmed.starts_with("BINARY=") {
                format!("BINARY=\"{project_name}\"")
            } else {
                line.to_string()
            }
        })
        .collect();
    let mut out = replaced.join("\n");
    if template.ends_with('\n') {
        out.push('\n');
    }
    out
}

#[test]
fn test_init_creates_all_files_and_dirs() {
    let dir = tempdir().unwrap();
    let base = dir.path();
    let project_name = "demo-project";

    create_in_dir(base, project_name).unwrap();

    assert!(exists(&base.join("agent-config")));
    assert!(exists(&base.join("agent-config/logs")));
    assert!(exists(&base.join("agent-config/query.txt")));
    assert!(exists(&base.join("src")));
    assert!(exists(&base.join(".gitignore")));
    assert!(exists(&base.join("build.sh")));
    assert!(exists(&base.join("Cargo.toml")));
    assert!(exists(&base.join("src/main.rs")));
    assert!(exists(&base.join("UserSpecification.md")));

    // Verify copied files match this repo's templates where specified
    let expected_gitignore = include_str!("../.gitignore");
    let expected_build_template = include_str!("../build.sh");
    let expected_build = replace_binary(expected_build_template, project_name);

    let got_gitignore = fs::read_to_string(base.join(".gitignore")).unwrap();
    let got_build = fs::read_to_string(base.join("build.sh")).unwrap();
    assert_eq!(got_gitignore, expected_gitignore);
    assert_eq!(got_build, expected_build);

    // Sanity check content of generated files
    let cargo = fs::read_to_string(base.join("Cargo.toml")).unwrap();
    assert!(cargo.contains("[package]"));
    assert!(cargo.contains("edition = \"2021\""));

    let main_rs = fs::read_to_string(base.join("src/main.rs")).unwrap();
    assert!(main_rs.contains("fn main()"));

    let user_spec = fs::read_to_string(base.join("UserSpecification.md")).unwrap();
    assert!(user_spec.contains("# User Specification"));

    let query_txt = fs::read_to_string(base.join("agent-config/query.txt")).unwrap();
    assert_eq!(query_txt, "");
}

#[test]
fn test_init_does_not_overwrite_existing_files() {
    let dir = tempdir().unwrap();
    let base = dir.path();
    let project_name = "custom-bin";

    // Pre-create some files with custom content
    fs::create_dir_all(base.join("src")).unwrap();
    fs::create_dir_all(base.join("agent-config")).unwrap();
    fs::write(base.join(".gitignore"), "custom-ignore\n").unwrap();
    fs::write(base.join("build.sh"), "#!/bin/bash\necho custom\n").unwrap();
    fs::write(base.join("Cargo.toml"), "[package]\nname = \"custom\"\n").unwrap();

    create_in_dir(base, project_name).unwrap();

    // Ensure pre-existing files are untouched
    let git = fs::read_to_string(base.join(".gitignore")).unwrap();
    let build = fs::read_to_string(base.join("build.sh")).unwrap();
    let cargo = fs::read_to_string(base.join("Cargo.toml")).unwrap();

    assert_eq!(git, "custom-ignore\n");
    assert_eq!(build, "#!/bin/bash\necho custom\n");
    assert!(cargo.contains("name = \"custom\""));

    // Ensure missing files were created
    assert!(base.join("UserSpecification.md").exists());
    assert!(base.join("agent-config").exists());
    assert!(base.join("agent-config/logs").exists());
    assert!(base.join("agent-config/query.txt").exists());
    assert!(base.join("src/main.rs").exists());
}

#[test]
fn test_init_is_idempotent() {
    let dir = tempdir().unwrap();
    let base = dir.path();

    create_in_dir(base, "first").unwrap();
    // Second call should not error and should not overwrite
    create_in_dir(base, "second").unwrap();

    // Validate still a sane structure
    assert!(base.join("src/main.rs").exists());
    assert!(base.join(".gitignore").exists());

    // If build.sh existed from first run, it should remain unchanged on second run
    let build = fs::read_to_string(base.join("build.sh")).unwrap();
    assert!(build.contains("BINARY=\"first\"") || build.contains("echo custom"));
}
