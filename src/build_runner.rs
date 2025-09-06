use crate::app_error::{AppError, BuildFailure};
use std::process::Command;

pub fn run() -> Result<String, BuildFailure> {
    let output = Command::new("bash")
        .arg("build.sh")
        .output()
        .expect("Failed to execute build.sh. Is bash installed and is build.sh executable?");

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let combined_output = format!("STDOUT:\n{stdout}\n\nSTDERR:\n{stderr}");

    if output.status.success() && stderr.trim().is_empty() {
        Ok(combined_output)
    } else {
        Err(BuildFailure {
            output: combined_output,
        })
    }
}

pub fn get_codebase_rollup() -> Result<String, AppError> {
    let output = Command::new("bash")
        .arg("codeRollup.sh")
        .arg("--all")
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::Config(format!("codeRollup.sh failed: {stderr}")));
    }

    // The rollup script writes to codeRollup.txt, so we read that file
    std::fs::read_to_string("codeRollup.txt").map_err(AppError::from)
}
