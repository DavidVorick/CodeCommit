use super::response_parser::parse_llm_response;
use std::path::PathBuf;

#[test]
fn preserves_internal_newlines() {
    let input = "\
^^^src/multi.rs
line1
line2
line3
^^^end
";
    let updates = parse_llm_response(input).expect("parser should succeed");
    assert_eq!(updates.len(), 1);
    assert_eq!(updates[0].path, PathBuf::from("src/multi.rs"));
    assert_eq!(updates[0].content.as_deref(), Some("line1\nline2\nline3"));
}

#[test]
fn preserves_duplicate_updates_for_same_path() {
    let input = "\
^^^src/dup.rs
first
^^^end
^^^src/dup.rs
second
^^^end
";
    let updates = parse_llm_response(input).expect("parser should succeed");
    assert_eq!(updates.len(), 2);
    assert_eq!(updates[0].path, PathBuf::from("src/dup.rs"));
    assert_eq!(updates[0].content.as_deref(), Some("first"));
    assert_eq!(updates[1].path, PathBuf::from("src/dup.rs"));
    assert_eq!(updates[1].content.as_deref(), Some("second"));
}

#[test]
fn unterminated_block_consumes_to_end() {
    let input = "\
^^^src/unterminated.rs
this block never ends
(no ^^^end)
and the parser will grab all remaining lines
";
    let updates = parse_llm_response(input).expect("parser should succeed");
    assert_eq!(updates.len(), 1);

    let content = updates[0].content.as_deref().unwrap();
    assert!(content.contains("this block never ends"));
    assert!(content.contains("grab all remaining lines"));
}
