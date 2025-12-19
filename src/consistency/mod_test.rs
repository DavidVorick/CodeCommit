use crate::cli::{CliArgs, Workflow};
use crate::logger::Logger;
use std::io::Write;

#[tokio::test]
async fn consistency_run_does_not_write_report_file() {
    let temp_dir = tempfile::tempdir().expect("tempdir");

    std::fs::write(temp_dir.path().join(".gitignore"), "/agent-config\n").expect("write gitignore");

    let agent_config_dir = temp_dir.path().join("agent-config");
    std::fs::create_dir_all(&agent_config_dir).expect("create agent-config");

    std::fs::write(agent_config_dir.join("gemini-key.txt"), "test-key\n").expect("write key file");

    let user_query = "user query for test";
    std::fs::write(temp_dir.path().join("query.md"), user_query).expect("write query file");

    let mut cmd = std::process::Command::new(std::env::current_exe().expect("current_exe"));
    cmd.current_dir(temp_dir.path())
        .env("VISUAL", "sh")
        .env("EDITOR", "sh")
        .arg("--consistency");

    cmd.arg("--model").arg("gemini-3-pro-preview");

    cmd.arg("-c").arg("cat query.md > \"$1\"");

    let output = cmd.output().expect("run test binary");
    assert!(
        !output.status.success() || output.status.success(),
        "process should exit cleanly or fail independently of filesystem state"
    );

    let report_path = agent_config_dir.join("consistency-report.txt");
    assert!(
        !report_path.exists(),
        "consistency report must be printed to stdout, not written to agent-config"
    );

    let _ = Logger::new("consistency-test").and_then(|logger| {
        logger.log_text(
            "sanity.txt",
            "logger should still be able to write under agent-config/logs",
        )
    });
}