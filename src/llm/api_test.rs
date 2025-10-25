use super::api::{extract_text_from_gemini_response, extract_text_from_gpt_response};
use crate::app_error::AppError;
use serde_json::json;

#[test]
fn test_extract_gemini_text_happy_path() {
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
    let result = extract_text_from_gemini_response(&response).unwrap();
    assert_eq!(result, "This is the LLM response.");
}

#[test]
fn test_extract_gemini_text_no_candidates() {
    let response = json!({ "candidates": [] });
    let result = extract_text_from_gemini_response(&response);
    assert!(matches!(result, Err(AppError::ResponseParsing(_))));
}

#[test]
fn test_extract_gemini_text_missing_candidates_key() {
    let response = json!({ "other_key": "value" });
    let result = extract_text_from_gemini_response(&response);
    assert!(matches!(result, Err(AppError::ResponseParsing(_))));
}

#[test]
fn test_extract_gemini_text_missing_parts() {
    let response = json!({
        "candidates": [
            {
                "content": {}
            }
        ]
    });
    let result = extract_text_from_gemini_response(&response);
    assert!(matches!(result, Err(AppError::ResponseParsing(_))));
}

#[test]
fn test_extract_gemini_text_missing_text() {
    let response = json!({
        "candidates": [
            {
                "content": {
                    "parts": [{ "not_text": "hello" }]
                }
            }
        ]
    });
    let result = extract_text_from_gemini_response(&response);
    assert!(matches!(result, Err(AppError::ResponseParsing(_))));
}

#[test]
fn test_extract_gemini_text_multiple_parts() {
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
    let result = extract_text_from_gemini_response(&response).unwrap();
    assert_eq!(
        result,
        "This is the first part. This is the second part. And this is the third."
    );
}

#[test]
fn test_extract_gpt_text_happy_path() {
    let response = json!({
        "choices": [
            {
                "message": {
                    "content": "This is the GPT response."
                }
            }
        ]
    });
    let result = extract_text_from_gpt_response(&response).unwrap();
    assert_eq!(result, "This is the GPT response.");
}

#[test]
fn test_extract_gpt_text_no_choices() {
    let response = json!({"choices": []});
    let result = extract_text_from_gpt_response(&response);
    assert!(matches!(result, Err(AppError::ResponseParsing(_))));
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Could not find 'content' in GPT response JSON."));
}

#[test]
fn test_extract_gpt_text_missing_content() {
    let response = json!({
        "choices": [
            {
                "message": {
                    "role": "assistant"
                }
            }
        ]
    });
    let result = extract_text_from_gpt_response(&response);
    assert!(matches!(result, Err(AppError::ResponseParsing(_))));
}

#[test]
fn test_extract_gpt_text_missing_message() {
    let response = json!({
        "choices": [
            {
                "finish_reason": "stop"
            }
        ]
    });
    let result = extract_text_from_gpt_response(&response);
    assert!(matches!(result, Err(AppError::ResponseParsing(_))));
}
