use crate::cli::{parse_args, CliArgs, Model, Workflow};

fn to_string_vec(args: &[&str]) -> Vec<String> {
    args.iter().map(|s| s.to_string()).collect()
}

#[test]
fn test_no_args() {
    let args = to_string_vec(&[]);
    let result = parse_args(args.into_iter()).unwrap();
    assert_eq!(
        result,
        CliArgs {
            model: Model::Gemini3Pro,
            workflow: Workflow::CommitCode,
            force: false,
            light_roll: false,
        }
    );
}

#[test]
fn test_model_arg() {
    let args_gpt5 = to_string_vec(&["--model", "gpt-5"]);
    let result_gpt5 = parse_args(args_gpt5.into_iter()).unwrap();
    assert_eq!(
        result_gpt5,
        CliArgs {
            model: Model::Gpt5,
            workflow: Workflow::CommitCode,
            force: false,
            light_roll: false,
        }
    );

    let args_gemini25 = to_string_vec(&["--model", "gemini-2.5-pro"]);
    let result_gemini25 = parse_args(args_gemini25.into_iter()).unwrap();
    assert_eq!(
        result_gemini25,
        CliArgs {
            model: Model::Gemini2_5Pro,
            workflow: Workflow::CommitCode,
            force: false,
            light_roll: false,
        }
    );

    let args_gemini3 = to_string_vec(&["--model", "gemini-3-pro"]);
    let result_gemini3 = parse_args(args_gemini3.into_iter()).unwrap();
    assert_eq!(
        result_gemini3,
        CliArgs {
            model: Model::Gemini3Pro,
            workflow: Workflow::CommitCode,
            force: false,
            light_roll: false,
        }
    );
}

#[test]
fn test_invalid_model() {
    let args = to_string_vec(&["--model", "gpt-4"]);
    let result = parse_args(args.into_iter());
    assert!(result.is_err());
}

#[test]
fn test_model_missing_value() {
    let args = to_string_vec(&["--model"]);
    let result = parse_args(args.into_iter());
    assert!(result.is_err());
}

#[test]
fn test_consistency_check_workflow() {
    let args = to_string_vec(&["--consistency-check"]);
    let result = parse_args(args.into_iter()).unwrap();
    assert_eq!(result.workflow, Workflow::ConsistencyCheck);

    let args = to_string_vec(&["--consistency"]);
    let result = parse_args(args.into_iter()).unwrap();
    assert_eq!(result.workflow, Workflow::ConsistencyCheck);

    let args = to_string_vec(&["--cc"]);
    let result = parse_args(args.into_iter()).unwrap();
    assert_eq!(result.workflow, Workflow::ConsistencyCheck);
}

#[test]
fn test_rollup_workflow() {
    let args = to_string_vec(&["--rollup"]);
    let result = parse_args(args.into_iter()).unwrap();
    assert_eq!(result.workflow, Workflow::Rollup);
    assert!(!result.light_roll);
}

#[test]
fn test_auto_workflow() {
    let args = to_string_vec(&["--aw"]);
    let result = parse_args(args.into_iter()).unwrap();
    assert_eq!(result.workflow, Workflow::Auto);
}

#[test]
fn test_force_flag() {
    let args = to_string_vec(&["--force"]);
    let result = parse_args(args.into_iter()).unwrap();
    assert_eq!(result.workflow, Workflow::CommitCode);
    assert!(result.force);

    let args = to_string_vec(&["--f"]);
    let result = parse_args(args.into_iter()).unwrap();
    assert_eq!(result.workflow, Workflow::CommitCode);
    assert!(result.force);
}

#[test]
fn test_force_with_consistency_is_error() {
    let args = to_string_vec(&["--f", "--cc"]);
    let result = parse_args(args.into_iter());
    assert!(result.is_err());
}

#[test]
fn test_force_with_rollup_is_error() {
    let args = to_string_vec(&["--f", "--rollup"]);
    let result = parse_args(args.into_iter());
    assert!(result.is_err());
}

#[test]
fn test_force_with_aw_is_error() {
    let args = to_string_vec(&["--f", "--aw"]);
    let result = parse_args(args.into_iter());
    assert!(result.is_err());
}

#[test]
fn test_multiple_workflows_is_error() {
    let args = to_string_vec(&["--cc", "--consistency"]);
    let result = parse_args(args.into_iter());
    assert!(result.is_err());

    let args = to_string_vec(&["--cc", "--rollup"]);
    let result = parse_args(args.into_iter());
    assert!(result.is_err());

    let args = to_string_vec(&["--rollup", "--consistency"]);
    let result = parse_args(args.into_iter());
    assert!(result.is_err());

    let args = to_string_vec(&["--aw", "--cc"]);
    let result = parse_args(args.into_iter());
    assert!(result.is_err());
}

#[test]
fn test_unknown_argument() {
    let args = to_string_vec(&["--unknown-arg"]);
    let result = parse_args(args.into_iter());
    assert!(result.is_err());
}

#[test]
fn test_model_and_force() {
    let args = to_string_vec(&["--model", "gpt-5", "--f"]);
    let result = parse_args(args.into_iter()).unwrap();
    assert_eq!(
        result,
        CliArgs {
            model: Model::Gpt5,
            workflow: Workflow::CommitCode,
            force: true,
            light_roll: false,
        }
    );

    let args = to_string_vec(&["--f", "--model", "gpt-5"]);
    let result = parse_args(args.into_iter()).unwrap();
    assert_eq!(
        result,
        CliArgs {
            model: Model::Gpt5,
            workflow: Workflow::CommitCode,
            force: true,
            light_roll: false,
        }
    );
}

#[test]
fn test_light_roll_with_rollup() {
    let args = to_string_vec(&["--rollup", "--light-roll"]);
    let result = parse_args(args.into_iter()).unwrap();
    assert_eq!(result.workflow, Workflow::Rollup);
    assert!(result.light_roll);

    let args = to_string_vec(&["--light-roll", "--rollup"]);
    let result = parse_args(args.into_iter()).unwrap();
    assert_eq!(result.workflow, Workflow::Rollup);
    assert!(result.light_roll);

    let args = to_string_vec(&["--rollup", "--lr"]);
    let result = parse_args(args.into_iter()).unwrap();
    assert_eq!(result.workflow, Workflow::Rollup);
    assert!(result.light_roll);
}

#[test]
fn test_light_roll_without_rollup_is_error() {
    let args = to_string_vec(&["--light-roll"]);
    let result = parse_args(args.into_iter());
    assert!(result.is_err());

    let args = to_string_vec(&["--lr"]);
    let result = parse_args(args.into_iter());
    assert!(result.is_err());
}

#[test]
fn test_light_roll_with_other_workflow_is_error() {
    let args = to_string_vec(&["--light-roll", "--cc"]);
    let result = parse_args(args.into_iter());
    assert!(result.is_err());

    let args = to_string_vec(&["--lr", "--cc"]);
    let result = parse_args(args.into_iter());
    assert!(result.is_err());
}