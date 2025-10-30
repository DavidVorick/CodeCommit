//! Happy-path tests for the `%files` response parser used by the context builder.

use super::response_parser::parse_context_llm_response;
use std::path::PathBuf;

#[test]
/// Parses a simple file list into PathBufs.
fn parses_basic_file_list() {
    let input = "\
%%%files
build.sh
Cargo.toml
src/main.rs
%%%end
";
    let files = parse_context_llm_response(input).expect("parser should succeed");
    assert_eq!(
        files,
        vec![
            PathBuf::from("build.sh"),
            PathBuf::from("Cargo.toml"),
            PathBuf::from("src/main.rs")
        ]
    );
}

#[test]
/// Trims whitespace and ignores blank lines within the block.
fn trims_and_ignores_blank_lines() {
    let input = "\
%%%files

  src/lib.rs  

src/thing.rs

%%%end
";
    let files = parse_context_llm_response(input).expect("parser should succeed");
    assert_eq!(
        files,
        vec![PathBuf::from("src/lib.rs"), PathBuf::from("src/thing.rs")]
    );
}

#[test]
/// Empty list is valid and returns an empty vector.
fn empty_list_is_valid() {
    let input = "\
%%%files
%%%end
";
    let files = parse_context_llm_response(input).expect("parser should succeed");
    assert!(files.is_empty());
}
