//! Tests covering the happy-path behavior of the `^^^` response parser.
//!
//! Scope: ensure we correctly parse single and multiple file updates,
//! empty-file creations, and deletes.

use super::response_parser::{parse_llm_response, FileUpdate};
use std::path::PathBuf;

#[test]
/// Parses a single file replacement with simple one-line content.
fn parses_single_file_replacement_minimal() {
    let input = "\
Preface text that the parser should ignore
^^^src/main.rs
fn main() {}
^^^end
Trailing notes are ignored too
";
    let updates = parse_llm_response(input).expect("parser should succeed");
    assert_eq!(updates.len(), 1);

    let FileUpdate { path, content } = &updates[0];
    assert_eq!(*path, PathBuf::from("src/main.rs"));
    assert_eq!(content.as_deref(), Some("fn main() {}"));
}

#[test]
/// Parses two sequential file blocks and preserves order.
fn parses_two_files_in_order() {
    let input = "\
^^^src/a.rs
pub fn a() {}
^^^end
^^^src/b.rs
pub fn b() {}
^^^end
";
    let updates = parse_llm_response(input).expect("parser should succeed");
    assert_eq!(updates.len(), 2);

    assert_eq!(updates[0].path, PathBuf::from("src/a.rs"));
    assert_eq!(updates[0].content.as_deref(), Some("pub fn a() {}"));

    assert_eq!(updates[1].path, PathBuf::from("src/b.rs"));
    assert_eq!(updates[1].content.as_deref(), Some("pub fn b() {}"));
}

#[test]
/// An empty block creates an empty file (content is Some("")).
fn creates_empty_file_block() {
    let input = "\
^^^src/lib.rs
^^^end
";
    let updates = parse_llm_response(input).expect("parser should succeed");
    assert_eq!(updates.len(), 1);
    assert_eq!(updates[0].path, PathBuf::from("src/lib.rs"));
    assert_eq!(updates[0].content.as_deref(), Some(""));
}

#[test]
/// A delete block produces a FileUpdate with `content = None`.
fn parses_delete_file_block() {
    let input = "\
^^^some/old_file.txt
^^^delete
";
    let updates = parse_llm_response(input).expect("parser should succeed");
    assert_eq!(updates.len(), 1);

    let FileUpdate { path, content } = &updates[0];
    assert_eq!(*path, PathBuf::from("some/old_file.txt"));
    assert!(content.is_none());
}

#[test]
/// Trailing blank lines before ^^^end are preserved as a single newline in the joined content.
fn preserves_trailing_blank_line_before_end() {
    let input = "\
^^^src/x.rs
line1

^^^end
";
    let updates = parse_llm_response(input).expect("parser should succeed");
    assert_eq!(updates.len(), 1);
    assert_eq!(updates[0].path, PathBuf::from("src/x.rs"));
    // `lines().join("\n")` yields "line1\n" for "line1\n\n^^^end"
    assert_eq!(updates[0].content.as_deref(), Some("line1\n"));
}
