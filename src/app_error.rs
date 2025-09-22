use std::fmt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Configuration Error: {0}")]
    Config(String),

    #[error("I/O Error: {0}")]
    Io(#[from] std::io::Error),

    #[error("HTTP Request Error: {0}")]
    Network(String),

    #[error("JSON Serialization/Deserialization Error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("LLM Response Parsing Error: {0}")]
    ResponseParsing(String),

    #[error("File Update Error: {0}")]
    FileUpdate(String),

    #[error("The build did not pass after the maximum number of attempts.")]
    MaxAttemptsReached,
}

#[derive(Debug)]
pub struct BuildFailure {
    pub output: String,
}

impl fmt::Display for BuildFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Build script failed with output:\n{}", self.output)
    }
}
