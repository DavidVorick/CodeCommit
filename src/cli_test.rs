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
            model: Model::Gemini2_5Pro,
            workflow: Workflow::CommitCode,
            refactor: false,
        }
    );
}

#[test]
fn test_model_arg() {
    let args = to_string_vec(&["--model", "gpt-5"]);
    let result = parse_args(args.into_iter()).unwrap();
    assert_eq!(
        result,
        CliArgs {
            model: Model::Gpt5,
            workflow: Workflow::CommitCode,
            refactor: false,
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
    assert!(!result.refactor);

    let args = to_string_vec(&["--consistency"]);
    let result = parse_args(args.into_iter()).unwrap();
    assert_eq!(result.workflow, Workflow::ConsistencyCheck);
    assert!(!result.refactor);

    let args = to_string_vec(&["--cc"]);
    let result = parse_args(args.into_iter()).unwrap();
    assert_eq!(result.workflow, Workflow::ConsistencyCheck);
    assert!(!result.refactor);
}

#[test]
fn test_refactor_flag() {
    let args = to_string_vec(&["--refactor"]);
    let result = parse_args(args.into_iter()).unwrap();
    assert_eq!(result.workflow, Workflow::CommitCode);
    assert!(result.refactor);

    let args = to_string_vec(&["--ref"]);
    let result = parse_args(args.into_iter()).unwrap();
    assert_eq!(result.workflow, Workflow::CommitCode);
    assert!(result.refactor);
}

#[test]
fn test_refactor_with_consistency_is_error() {
    let args = to_string_vec(&["--ref", "--cc"]);
    let result = parse_args(args.into_iter());
    assert!(result.is_err());
}

#[test]
fn test_multiple_workflows_is_error() {
    let args = to_string_vec(&["--cc", "--consistency"]);
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
fn test_model_and_refactor() {
    let args = to_string_vec(&["--model", "gpt-5", "--ref"]);
    let result = parse_args(args.into_iter()).unwrap();
    assert_eq!(
        result,
        CliArgs {
            model: Model::Gpt5,
            workflow: Workflow::CommitCode,
            refactor: true,
        }
    );

    let args = to_string_vec(&["--ref", "--model", "gpt-5"]);
    let result = parse_args(args.into_iter()).unwrap();
    assert_eq!(
        result,
        CliArgs {
            model: Model::Gpt5,
            workflow: Workflow::CommitCode,
            refactor: true,
        }
    );
}
