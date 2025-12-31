use super::executor;
use crate::auto_workflow::types::Stage;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_validate_response_format_success() {
    let response = "Some text\n@@@@task-success@@@@\nMore text";
    assert!(executor::validate_response_format(response).is_ok());
}

#[test]
fn test_validate_response_format_failure_no_tag() {
    let response = "Some text\nNo tag here";
    assert!(executor::validate_response_format(response).is_err());
}

#[test]
fn test_validate_response_format_failure_multiple_tags() {
    let response = "@@@@task-success@@@@\n@@@@changes-requested@@@@";
    assert!(executor::validate_response_format(response).is_err());
}

#[test]
fn test_validate_response_format_comments_valid() {
    let response = "@@@@task-success@@@@\n%%%%comment%%%%\nHello\n%%%%end%%%%";
    assert!(executor::validate_response_format(response).is_ok());
}

#[test]
fn test_validate_response_format_comments_invalid_multiple() {
    let response =
        "@@@@task-success@@@@\n%%%%comment%%%%\nA\n%%%%end%%%%\n%%%%comment%%%%\nB\n%%%%end%%%%";
    assert!(executor::validate_response_format(response).is_err());
}

#[test]
fn test_validate_response_format_comments_invalid_mismatch() {
    let response = "@@@@task-success@@@@\n%%%%comment%%%%\nA";
    assert!(executor::validate_response_format(response).is_err());
}

#[test]
fn test_extract_comment() {
    let response = "Prefix\n%%%%comment%%%%\n  This is a comment.  \n%%%%end%%%%\nSuffix";
    let comment = executor::extract_comment(response);
    assert!(comment.is_some());
    assert_eq!(comment.unwrap().trim(), "This is a comment.");

    let response_no_comment = "Just text";
    assert!(executor::extract_comment(response_no_comment).is_none());
}

#[test]
fn test_mark_stage_complete() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let mod_dir = root.join("src/mod");
    fs::create_dir_all(&mod_dir).unwrap();
    let spec_path = mod_dir.join("UserSpecification.md");
    let content = "SPEC CONTENT";

    executor::mark_stage_complete(root, &spec_path, Stage::Implemented, content).unwrap();

    let state_path = root.join("agent-state/specifications/src/mod/implemented");
    assert!(state_path.exists());
    assert_eq!(fs::read_to_string(state_path).unwrap(), content);
}
