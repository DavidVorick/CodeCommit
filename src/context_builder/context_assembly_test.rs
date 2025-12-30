use super::load_files_with_root;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_load_files_happy_path() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    // Create a .gitignore to satisfy PathFilter
    fs::write(root.join(".gitignore"), "").unwrap();

    // Create some dummy files
    let file1 = root.join("src/main.rs");
    fs::create_dir_all(file1.parent().unwrap()).unwrap();
    fs::write(&file1, "fn main() {}\n").unwrap();

    let file2 = root.join("README.md");
    fs::write(&file2, "# Readme").unwrap(); // No trailing newline

    let paths = vec![PathBuf::from("src/main.rs"), PathBuf::from("README.md")];

    let result = load_files_with_root(paths, root).expect("should succeed");

    // Check content
    // Note: The function appends a newline if missing
    let expected_part1 = "--- src/main.rs ---\nfn main() {}\n\n";
    let expected_part2 = "--- README.md ---\n# Readme\n\n";

    assert!(result.contains(expected_part1));
    assert!(result.contains(expected_part2));
}

#[test]
fn test_load_files_respects_gitignore() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    // Ignore secret.txt
    fs::write(root.join(".gitignore"), "secret.txt").unwrap();

    let secret = root.join("secret.txt");
    fs::write(&secret, "hidden").unwrap();

    let paths = vec![PathBuf::from("secret.txt")];

    let result = load_files_with_root(paths, root);

    // Should fail because validation fails for ignored file
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("matches a rule in .gitignore"));
}
