use super::response_parser::parse_context_llm_response;
use crate::app_error::AppError;

#[test]
fn errors_when_no_markers_present() {
    let input = "no markers in here";
    let err = parse_context_llm_response(input).unwrap_err();
    match err {
        AppError::ResponseParsing(msg) => {
            assert!(
                msg.contains("Could not find '%%%files'...'%%%end' block"),
                "unexpected msg: {msg}"
            );
        }
        other => panic!("expected ResponseParsing, got {other}"),
    }
}

#[test]
fn errors_when_end_without_files() {
    let input = "%%%end";
    let err = parse_context_llm_response(input).unwrap_err();
    match err {
        AppError::ResponseParsing(msg) => {
            assert!(
                msg.contains("Found '%%%end' without a preceding '%%%files'"),
                "unexpected msg: {msg}"
            );
        }
        other => panic!("expected ResponseParsing, got {other}"),
    }
}

#[test]
fn errors_when_files_without_end() {
    let input = "\
%%%files
src/main.rs
";
    let err = parse_context_llm_response(input).unwrap_err();
    match err {
        AppError::ResponseParsing(msg) => {
            assert!(
                msg.contains("Found '%%%files' but no matching '%%%end'"),
                "unexpected msg: {msg}"
            );
        }
        other => panic!("expected ResponseParsing, got {other}"),
    }
}
