use crate::app_error::AppError;
use crate::response_parser::{parse_llm_response, FileUpdate};
use std::path::PathBuf;

#[test]
fn test_parse_llm_response_happy_path() {
    let response_text = r#"
Some introductory text from the LLM.

^^^src/main.rs
fn main() {
    println!("Hello, world!");
}
^^^end

Some concluding text.

^^^src/lib.rs
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
^^^end
"#;

    let expected = vec![
        FileUpdate {
            path: PathBuf::from("src/main.rs"),
            content: Some("fn main() {\n    println!(\"Hello, world!\");\n}".to_string()),
        },
        FileUpdate {
            path: PathBuf::from("src/lib.rs"),
            content: Some("pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}".to_string()),
        },
    ];

    let result = parse_llm_response(response_text).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_parse_llm_response_create_empty_file() {
    let response_text = r#"
^^^src/empty.rs
^^^end
"#;

    let expected = vec![FileUpdate {
        path: PathBuf::from("src/empty.rs"),
        content: Some("".to_string()),
    }];

    let result = parse_llm_response(response_text).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_parse_llm_response_with_deletion() {
    let response_text = r#"
^^^src/obsolete.rs
^^^delete
"#;
    let expected = vec![FileUpdate {
        path: PathBuf::from("src/obsolete.rs"),
        content: None,
    }];
    let result = parse_llm_response(response_text).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_empty_input() {
    let result = parse_llm_response("").unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_no_file_blocks() {
    let response_text = "Just some text from the LLM, no code blocks.";
    let result = parse_llm_response(response_text).unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_block_with_no_end_marker() {
    let response_text = r#"
^^^src/main.rs
fn main() {
    // This file is not terminated correctly
"#;
    let expected = vec![FileUpdate {
        path: PathBuf::from("src/main.rs"),
        content: Some("fn main() {\n    // This file is not terminated correctly".to_string()),
    }];
    let result = parse_llm_response(response_text).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_block_with_no_filename() {
    let response_text = r#"
^^^
fn main() {}
^^^end
"#;
    let result = parse_llm_response(response_text);
    assert!(matches!(result, Err(AppError::ResponseParsing(_))));
    assert_eq!(
        result.unwrap_err().to_string(),
        "LLM Response Parsing Error: Found '^^^' without a filename."
    );
}

#[test]
fn test_mixed_create_delete_and_empty() {
    let response_text = r#"
^^^src/new.rs
pub const VALUE: i32 = 42;
^^^end

^^^src/old.rs
^^^delete

^^^src/empty.rs
^^^end
"#;
    let expected = vec![
        FileUpdate {
            path: PathBuf::from("src/new.rs"),
            content: Some("pub const VALUE: i32 = 42;".to_string()),
        },
        FileUpdate {
            path: PathBuf::from("src/old.rs"),
            content: None,
        },
        FileUpdate {
            path: PathBuf::from("src/empty.rs"),
            content: Some("".to_string()),
        },
    ];
    let result = parse_llm_response(response_text).unwrap();
    assert_eq!(result, expected);
}
