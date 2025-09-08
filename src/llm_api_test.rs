use crate::app_error::AppError;
use crate::llm_api::extract_text_from_response;
use serde_json::json;

#[test]
fn test_extract_text_happy_path() {
    let response = json!({
        "candidates": [
            {
                "content": {
                    "parts": [
                        {
                            "text": "This is the LLM response."
                        }
                    ]
                }
            }
        ]
    });
    let result = extract_text_from_response(&response).unwrap();
    assert_eq!(result, "This is the LLM response.");
}

#[test]
fn test_extract_text_no_candidates() {
    let response = json!({ "candidates": [] });
    let result = extract_text_from_response(&response);
    assert!(matches!(result, Err(AppError::ResponseParsing(_))));
}

#[test]
fn test_extract_text_missing_candidates_key() {
    let response = json!({ "other_key": "value" });
    let result = extract_text_from_response(&response);
    assert!(matches!(result, Err(AppError::ResponseParsing(_))));
}

#[test]
fn test_extract_text_missing_parts() {
    let response = json!({
        "candidates": [
            {
                "content": {}
            }
        ]
    });
    let result = extract_text_from_response(&response);
    assert!(matches!(result, Err(AppError::ResponseParsing(_))));
}

#[test]
fn test_extract_text_missing_text() {
    let response = json!({
        "candidates": [
            {
                "content": {
                    "parts": [{ "not_text": "hello" }]
                }
            }
        ]
    });
    let result = extract_text_from_response(&response);
    assert!(matches!(result, Err(AppError::ResponseParsing(_))));
}

#[test]
fn test_extract_text_multiple_parts() {
    let response = json!({
        "candidates": [
            {
                "content": {
                    "parts": [
                        {
                            "text": "This is the first part. "
                        },
                        {
                            "text": "This is the second part. "
                        },
                        {
                            "text": "And this is the third."
                        }
                    ]
                }
            }
        ]
    });
    let result = extract_text_from_response(&response).unwrap();
    assert_eq!(
        result,
        "This is the first part. This is the second part. And this is the third."
    );
}
