use crate::app_error::AppError;
use std::fs;
use std::path::Path;

const TEMPLATE_DOT_GITIGNORE: &str = include_str!("../.gitignore");
const TEMPLATE_BUILD_SH: &str = include_str!("../build.sh");

const TEMPLATE_SRC_MAIN_RS: &str = r#"fn main() {
    println!("Hello from your new CodeCommit project!");
}
"#;

const TEMPLATE_USER_SPEC: &str = r#"# User Specification

Describe your project here. This document defines your project's requirements and must remain the source of truth for your implementation.
"#;

pub fn run_init_command(project_name: &str) -> Result<(), AppError> {
    create_in_dir(Path::new("."), project_name)
}

pub fn create_in_dir(base_dir: &Path, project_name: &str) -> Result<(), AppError> {
    ensure_dir(&base_dir.join("agent-config"))?;
    ensure_dir(&base_dir.join("agent-config/logs"))?;
    ensure_dir(&base_dir.join("src"))?;

    write_if_missing(&base_dir.join(".gitignore"), TEMPLATE_DOT_GITIGNORE)?;
    write_if_missing(&base_dir.join("build.sh"), TEMPLATE_BUILD_SH)?;
    write_if_missing(
        &base_dir.join("Cargo.toml"),
        &cargo_toml_with_name(project_name),
    )?;
    write_if_missing(&base_dir.join("src/main.rs"), TEMPLATE_SRC_MAIN_RS)?;
    write_if_missing(&base_dir.join("UserSpecification.md"), TEMPLATE_USER_SPEC)?;
    write_if_missing(&base_dir.join("agent-config/query.txt"), "")?;
    Ok(())
}

fn ensure_dir(path: &Path) -> Result<(), AppError> {
    fs::create_dir_all(path)?;
    Ok(())
}

fn write_if_missing(path: &Path, content: &str) -> Result<bool, AppError> {
    if path.exists() {
        return Ok(false);
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)?;
    Ok(true)
}

fn cargo_toml_with_name(project_name: &str) -> String {
    format!(
        r#"[package]
name = "{project_name}"
version = "0.1.0"
edition = "2021"

[dependencies]
"#
    )
}
