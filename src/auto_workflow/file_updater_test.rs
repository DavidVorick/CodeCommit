use super::file_updater;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_apply_file_updates_create_and_update() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();

    let file_path = root.join("test.txt");
    let file_str = file_path.to_str().unwrap();

    // Create file
    let create_payload = format!("^^^{file_str}\nHello World\n^^^end");

    file_updater::apply_file_updates(&create_payload).unwrap();
    assert_eq!(fs::read_to_string(&file_path).unwrap(), "Hello World\n");

    // Update file
    let update_payload = format!("^^^{file_str}\nUpdated Content\n^^^end");

    file_updater::apply_file_updates(&update_payload).unwrap();
    assert_eq!(fs::read_to_string(&file_path).unwrap(), "Updated Content\n");
}

#[test]
fn test_apply_file_updates_delete() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();

    let file_path = root.join("delete_me.txt");
    let file_str = file_path.to_str().unwrap();

    fs::write(&file_path, "To be deleted").unwrap();
    assert!(file_path.exists());

    let delete_payload = format!("^^^{file_str}\n^^^delete\n^^^end");

    file_updater::apply_file_updates(&delete_payload).unwrap();
    assert!(!file_path.exists());
}

#[test]
fn test_apply_file_updates_multiple() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();

    let file1 = root.join("file1.txt");
    let file2 = root.join("file2.txt");

    let payload = format!(
        "^^^{}\nContent 1\n^^^end\n^^^{}\nContent 2\n^^^end",
        file1.to_str().unwrap(),
        file2.to_str().unwrap()
    );

    file_updater::apply_file_updates(&payload).unwrap();

    assert_eq!(fs::read_to_string(file1).unwrap(), "Content 1\n");
    assert_eq!(fs::read_to_string(file2).unwrap(), "Content 2\n");
}

#[test]
fn test_apply_file_updates_nested_dir() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();

    let nested_file = root.join("dir/nested/file.txt");

    let payload = format!("^^^{}\nNested\n^^^end", nested_file.to_str().unwrap());

    file_updater::apply_file_updates(&payload).unwrap();
    assert_eq!(fs::read_to_string(nested_file).unwrap(), "Nested\n");
}
