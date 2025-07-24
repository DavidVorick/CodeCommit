use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Configuration Error: {0}")]
    Config(String),
    #[error("I/O Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("HTTP Request Error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("JSON Deserialization Error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("LLM API Error: {0}")]
    Api(String),
}