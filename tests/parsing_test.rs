use implement::parsing;
use std::collections::HashMap;
use std::path::PathBuf;

#[test]
fn test_parse_happy_path_with_file_changes() {
    let input = r#"
&&&start
This is a thought for the user.
&&&end
%%%start
This is a debug thought.
%%%end
^^^src/main.rs
fn main() {
    println!("hello");
}
^^^end
^^^src/lib.rs
pub fn new() {}
^^^end
"#;
    let result = parsing::parse(input).unwrap();

    assert_eq!(result.user_thoughts, vec!["This is a thought for the user."]);
    assert_eq!(result.debug_thoughts, vec!["This is a debug thought."]);
    assert_eq!(result.file_changes.len(), 2);
    assert_eq!(
        result.file_changes.get(&PathBuf::from("src/main.rs")).unwrap(),
        "fn main() {\n    println!(\"hello\");\n}"
    );
    assert_eq!(
        result.file_changes.get(&PathBuf::from("src/lib.rs")).unwrap(),
        "pub fn new() {}"
    );
    assert!(result.success_message.is_none());
}

#[test]
fn test_parse_happy_path_with_success_signal() {
    let input = r#"
&&&start
I've analyzed the code and no changes are necessary.
&&&end
$$$start
No changes needed, build is already perfect.
$$$end
"#;
    let result = parsing::parse(input).unwrap();

    assert_eq!(result.user_thoughts.len(), 1);
    assert!(result.debug_thoughts.is_empty());
    assert!(result.file_changes.is_empty());
    assert_eq!(
        result.success_message,
        Some("No changes needed, build is already perfect.".to_string())
    );
}

#[test]
fn test_parse_error_on_both_files_and_success() {
    let input = r#"
^^^src/main.rs
fn main() {}
^^^end
$$$start
This should not be here.
$$$end
"#;
    let result = parsing::parse(input);
    assert!(result.is_err());
    assert_eq!(
        result.err().unwrap(),
        "Parsing error: Response cannot contain both file changes (^^^) and a success signal ($$$)."
    );
}

#[test]
fn test_parse_error_on_neither_files_nor_success() {
    let input = r#"
&&&start
Just a thought.
&&&end
%%%start
Just a debug thought.
%%%end
"#;
    let result = parsing::parse(input);
    assert!(result.is_err());
    assert_eq!(
        result.err().unwrap(),
        "Parsing error: Response must contain either file changes (^^^) or a success signal ($$$)."
    );
}

#[test]
fn test_parse_error_on_path_traversal() {
    let input = r#"
^^^../security/secrets.txt
TOP_SECRET_KEY=123
^^^end
"#;
    let result = parsing::parse(input);
    assert!(result.is_err());
    assert!(result
        .err()
        .unwrap()
        .contains("Path traversal ('..' or absolute paths) is not allowed"));
}

#[test]
fn test_parse_error_on_git_directory_access() {
    let input = r#"
^^^.git/config
[core]
    repositoryformatversion = 0
^^^end
"#;
    let result = parsing::parse(input);
    assert!(result.is_err());
    assert_eq!(result.err().unwrap(), "Access to the .git directory is forbidden");
}

#[test]
fn test_parse_error_on_duplicate_file() {
    let input = r#"
^^^src/main.rs
fn one() {}
^^^end
^^^src/main.rs
fn two() {}
^^^end
"#;
    let result = parsing::parse(input);
    assert!(result.is_err());
    assert_eq!(
        result.err().unwrap(),
        "Duplicate file modification detected for: src/main.rs"
    );
}

#[test]
fn test_parse_error_on_unterminated_block() {
    let input = r#"
^^^src/main.rs
fn main() {
    println!("hello");
}
"#; // Missing ^^^end
    let result = parsing::parse(input);
    assert!(result.is_err());
    assert!(result.err().unwrap().contains("Malformed response: Unparsed content remains."));
}

#[test]
fn test_format_file_changes_for_prompt() {
    let mut file_changes = HashMap::new();
    file_changes.insert(PathBuf::from("src/main.rs"), "fn main() {}".to_string());
    file_changes.insert(
        PathBuf::from("tests/a_test.rs"),
        "#[test] fn t() {}".to_string(),
    );

    let formatted = parsing::format_file_changes_for_prompt(&file_changes);

    let expected1 = "^^^src/main.rs\nfn main() {}\n^^^end\n";
    let expected2 = "^^^tests/a_test.rs\n#[test] fn t() {}\n^^^end\n";

    assert!(formatted.contains(expected1));
    assert!(formatted.contains(expected2));
    assert_eq!(formatted.len(), expected1.len() + expected2.len());
}