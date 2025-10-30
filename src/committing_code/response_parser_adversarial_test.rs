//! Adversarial and robustness tests for `^^^` parser.

use super::response_parser::parse_llm_response;
use std::path::PathBuf;

#[test]
/// Stray control lines outside a file header are ignored.
fn ignores_stray_delete_and_end_lines() {
    let input = "\
^^^delete
some random line
^^^end
^^^src/file.rs
ok
^^^end
";
    let updates = parse_llm_response(input).expect("parser should succeed");
    assert_eq!(updates.len(), 1);
    assert_eq!(updates[0].path, PathBuf::from("src/file.rs"));
    assert_eq!(updates[0].content.as_deref(), Some("ok"));
}

#[test]
/// The sentinel only matches when the line equals "^^^end".
/// Embedded text like `println!(\"^^^end\");` should not terminate the block.
fn embedded_end_text_does_not_terminate() {
    let input = "\
^^^src/embedded.rs
println!(\"^^^end\");
still inside the block
^^^end
";
    let updates = parse_llm_response(input).expect("parser should succeed");
    assert_eq!(updates.len(), 1);
    let content = updates[0].content.as_deref().unwrap();
    assert!(content.contains("println!(\"^^^end\");"));
    assert!(content.contains("still inside the block"));
}

#[test]
/// Parser does not validate paths (security is enforced by file_updater).
/// Document that raw paths—including traversal attempts—are parsed verbatim.
fn parses_path_verbatim_even_if_suspicious() {
    let input = "\
^^^../etc/passwd
x
^^^end
";
    let updates = parse_llm_response(input).expect("parser should succeed");
    assert_eq!(updates.len(), 1);
    assert_eq!(updates[0].path, PathBuf::from("../etc/passwd"));
    assert_eq!(updates[0].content.as_deref(), Some("x"));
}
