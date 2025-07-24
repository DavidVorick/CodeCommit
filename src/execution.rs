use anyhow::{Context, Result};
use std::process::Stdio;
use tokio::process::Command;

pub struct BuildResult {
    pub success: bool,
    pub output: String,
}

pub async fn run_build_script() -> Result<BuildResult> {
    let output = Command::new("./build.sh")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .context("Failed to execute build.sh")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    let combined_output = format!("STDOUT:\n{}\n\nSTDERR:\n{}", stdout, stderr);

    Ok(BuildResult {
        success: output.status.success(),
        output: combined_output,
    })
}