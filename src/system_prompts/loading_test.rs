use super::*;

#[test]
fn test_prompts_are_loaded_and_non_empty() {
    assert!(!CODE_MODIFICATION_INSTRUCTIONS.trim().is_empty());
    assert!(!CONTEXT_BUILDER_CONTEXT_QUERY.trim().is_empty());
    assert!(!COMMITTING_CODE_INITIAL_QUERY.trim().is_empty());
    assert!(!COMMITTING_CODE_REPAIR_QUERY.trim().is_empty());
    assert!(!COMMITTING_CODE_EXTRA_CODE_QUERY.trim().is_empty());
    assert!(!COMMITTING_CODE_REFACTOR_QUERY.trim().is_empty());
    assert!(!CONSISTENCY_CHECK.trim().is_empty());
    assert!(!PROJECT_STRUCTURE.trim().is_empty());
}
