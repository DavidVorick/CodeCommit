use super::*;
use std::io;

#[test]
fn test_config_error_display() {
    let err = AppError::Config("missing file".to_string());
    assert_eq!(err.to_string(), "Configuration Error: missing file");
}

#[test]
fn test_io_error_display() {
    let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
    let err = AppError::Io(io_err);
    let msg = err.to_string();
    assert!(msg.starts_with("I/O Error: "));
    // The exact error message from std::io::Error depends on the OS, but usually contains the string provided.
    assert!(msg.contains("file not found"));
}

#[test]
fn test_network_error_display() {
    let err = AppError::Network("timeout".to_string());
    assert_eq!(err.to_string(), "HTTP Request Error: timeout");
}

#[test]
fn test_json_error_display() {
    // Generate a real serde_json error
    let err_result: Result<serde_json::Value, _> = serde_json::from_str("{invalid");
    let json_err = err_result.unwrap_err();
    let err = AppError::Json(json_err);
    assert!(err.to_string().starts_with("JSON Serialization/Deserialization Error: "));
}

#[test]
fn test_response_parsing_error_display() {
    let err = AppError::ResponseParsing("bad format".to_string());
    assert_eq!(err.to_string(), "LLM Response Parsing Error: bad format");
}

#[test]
fn test_file_update_error_display() {
    let err = AppError::FileUpdate("access denied".to_string());
    assert_eq!(err.to_string(), "File Update Error: access denied");
}

#[test]
fn test_max_attempts_error_display() {
    let err = AppError::MaxAttemptsReached;
    assert_eq!(
        err.to_string(),
        "The build did not pass after the maximum number of attempts."
    );
}

#[test]
fn test_build_failure_display() {
    let err = BuildFailure {
        output: "compilation failed".to_string(),
    };
    assert_eq!(
        err.to_string(),
        "Build script failed with output:\ncompilation failed"
    );
}
