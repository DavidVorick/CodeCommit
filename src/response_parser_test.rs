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
            content: "fn main() {\n    println!(\"Hello, world!\");\n}".to_string(),
        },
        FileUpdate {
            path: PathBuf::from("src/lib.rs"),
            content: "pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}".to_string(),
        },
    ];

    let result = parse_llm_response(response_text).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_parse_llm_response_with_deletion() {
    let response_text = r#"
^^^src/obsolete.rs
^^^end
"#;

    let expected = vec![FileUpdate {
        path: PathBuf::from("src/obsolete.rs"),
        content: "".to_string(),
    }];

    let result = parse_llm_response(response_text).unwrap();
    assert_eq!(result, expected);
}
