use implement::{config::Config, state::{History, Interaction}};

#[test]
fn test_history_build_prompt() {
    let config = Config {
        api_key: "dummy".to_string(),
        system_prompt: "SYS_PROMPT".to_string(),
        basic_query: "BASIC_QUERY".to_string(),
        codebase_context: "CODEBASE".to_string(),
    };

    // Test initial prompt
    let history = History::new(config.clone());
    let initial_prompt = history.build_prompt();
    assert_eq!(initial_prompt, "SYS_PROMPT\n\nBASIC_QUERY\n\nCODEBASE");

    // Test prompt after one interaction
    let mut history = History::new(config.clone());
    let interaction1 = Interaction {
        debug_thoughts: "DEBUG_1".to_string(),
        file_changes: "FILES_1".to_string(),
        build_output: "BUILD_OUTPUT_1".to_string(),
    };
    history.add_interaction(interaction1);

    let prompt1 = history.build_prompt();
    let expected1 = "SYS_PROMPT\n\nBASIC_QUERY\n\nCODEBASE\n\nThe above query was provided, and you provided the following data in your response:\nDEBUG_1\nFILES_1\nWhen the file replacements were made, the build provided the following output:\nBUILD_OUTPUT_1";
    assert_eq!(prompt1, expected1);

    // Test prompt after a second interaction
    let interaction2 = Interaction {
        debug_thoughts: "DEBUG_2".to_string(),
        file_changes: "FILES_2".to_string(),
        build_output: "BUILD_OUTPUT_2".to_string(),
    };
    history.add_interaction(interaction2);

    let prompt2 = history.build_prompt();
    let expected2 = format!("{}\n\nYou then provided the following data in your subsequent response:\nDEBUG_2\nFILES_2\nWhen the file replacements were made, the build provided the following output:\nBUILD_OUTPUT_2", expected1);
    assert_eq!(prompt2, expected2);
}