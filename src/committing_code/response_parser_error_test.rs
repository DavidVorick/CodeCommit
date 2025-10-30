//! Tests for the explicit error conditions in the `^^^` response parser.

use super::response_parser::parse_llm_response;
use crate::app_error::AppError;

#[test]
/// A line with exactly "^^^" and no filename must error.
fn errors_when_filename_is_missing() {
    let input = "\
^^^
some content
^^^end
";
    let err = parse_llm_response(input).unwrap_err();
    match err {
        AppError::ResponseParsing(msg) => {
            assert!(
                msg.contains("Found '^^^' without a filename"),
                "unexpected message: {msg}"
            );
        }
        other => panic!("expected ResponseParsing, got {other}"),
    }
}
