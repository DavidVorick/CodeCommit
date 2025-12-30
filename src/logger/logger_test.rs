use super::Logger;
use serde_json::json;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

fn generate_test_suffix() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("test-{}", now.as_nanos())
}

#[test]
fn test_logger_happy_path() {
    // 1. Setup with unique suffix to avoid collision
    let suffix = generate_test_suffix();
    let logger = Logger::new(&suffix).expect("Failed to create Logger");

    // 2. Log Text
    let text_file = "test_note.txt";
    let text_content = "This is a test log entry.";
    logger
        .log_text(text_file, text_content)
        .expect("Failed to log text");

    // 3. Log JSON
    let json_file = "test_data.json";
    let json_content = json!({
        "status": "success",
        "attempt": 1
    });
    logger
        .log_json(json_file, &json_content)
        .expect("Failed to log JSON");

    // 4. Verification
    // We need to locate the directory created by the logger.
    // It should be in agent-config/logs and end with our suffix.
    let logs_root = Path::new("agent-config").join("logs");
    let mut target_dir = None;

    if let Ok(entries) = fs::read_dir(&logs_root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.ends_with(&suffix) {
                        target_dir = Some(path);
                        break;
                    }
                }
            }
        }
    }

    let target_dir = target_dir.expect("Failed to find the log directory matching the test suffix");

    // Verify text file content
    let text_path = target_dir.join(text_file);
    assert!(text_path.exists(), "Text log file does not exist");
    let stored_text = fs::read_to_string(text_path).expect("Failed to read text log file");
    assert_eq!(stored_text, text_content);

    // Verify JSON file content
    let json_path = target_dir.join(json_file);
    assert!(json_path.exists(), "JSON log file does not exist");
    let stored_json = fs::read_to_string(json_path).expect("Failed to read JSON log file");
    let expected_json = serde_json::to_string_pretty(&json_content).unwrap();
    assert_eq!(stored_json, expected_json);
}
