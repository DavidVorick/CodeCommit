use super::build_runner;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_run_build_success() {
    let dir = tempdir().unwrap();
    let build_script = dir.path().join("build.sh");
    // Ensure the script is valid bash that exits 0
    fs::write(&build_script, "#!/bin/bash\necho 'Hello World'\nexit 0").unwrap();

    let result = build_runner::run(dir.path());
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("EXIT CODE: 0"));
    assert!(output.contains("STDOUT:\nHello World"));
}

#[test]
fn test_run_build_failure() {
    let dir = tempdir().unwrap();
    let build_script = dir.path().join("build.sh");
    // Ensure the script exits with non-zero code and writes to stderr
    fs::write(&build_script, "#!/bin/bash\n>&2 echo 'Some Error'\nexit 1").unwrap();

    let result = build_runner::run(dir.path());
    assert!(result.is_err());
    let failure = result.unwrap_err();
    assert!(failure.output.contains("EXIT CODE: 1"));
    assert!(failure.output.contains("STDERR:\nSome Error"));
}
