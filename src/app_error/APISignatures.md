# API Signatures

pub enum AppError {
    Config(String),
    Io(std::io::Error),
    Network(String),
    Json(serde_json::Error),
    ResponseParsing(String),
    FileUpdate(String),
    MaxAttemptsReached,
}

pub struct BuildFailure {
    pub output: String,
}