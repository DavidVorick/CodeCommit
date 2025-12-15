use super::response_parser::parse_extra_files_response;
use std::path::PathBuf;

#[test]
fn parses_files_block() {
    let input = "
Some text
%%%files
src/foo.rs
src/bar/baz.rs
%%%end
More text
";
    let files = parse_extra_files_response(input).expect("should parse");
    assert_eq!(files.len(), 2);
    assert_eq!(files[0], PathBuf::from("src/foo.rs"));
    assert_eq!(files[1], PathBuf::from("src/bar/baz.rs"));
}

#[test]
fn handles_empty_block() {
    let input = "%%%files\n%%%end";
    let files = parse_extra_files_response(input).expect("should parse");
    assert!(files.is_empty());
}

#[test]
fn ignores_whitespace() {
    let input = "%%%files  \n  src/a.rs  \n  %%%end";
    let files = parse_extra_files_response(input).expect("should parse");
    assert_eq!(files.len(), 1);
    assert_eq!(files[0], PathBuf::from("src/a.rs"));
}
