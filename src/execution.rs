use anyhow::Result;
use std::process::Stdio;
use tokio::process::Command;

pub struct BuildResult {
    pub success: bool,
    pub output: String,
}

pub async fn run_build_script() -> Result<BuildResult> {
    let output_result = Command::new("./build.sh")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await;

    match output_result {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let combined_output = format!("STDOUT:\n{}\n\nSTDERR:\n{}", stdout, stderr);
            Ok(BuildResult {
                success: output.status.success(),
                output: combined_output,
            })
        }
        Err(e) => {
            // If the script fails to execute, treat it as a build failure.
            // This prevents the whole application from crashing.
            let error_output = format!(
                "Failed to execute build.sh. The script might not exist, may not be executable, or it might have produced too much output for the system's memory.\n\nError: {}",
                e
            );
            Ok(BuildResult {
                success: false,
                output: error_output,
            })
        }
    }
}
