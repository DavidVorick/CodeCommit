use super::file_updater;

#[test]
fn test_has_pending_updates_detection() {
    let content_no_updates = "This response has no updates.";
    assert!(!file_updater::has_pending_updates(content_no_updates));

    let content_with_updates = "Here is an update:\n^^^src/main.rs\nfn main() {}\n^^^end";
    assert!(file_updater::has_pending_updates(content_with_updates));

    let content_broken_update = "Here is a broken update:\n^^^src/main.rs\nfn main() {}";
    assert!(!file_updater::has_pending_updates(content_broken_update));
}
