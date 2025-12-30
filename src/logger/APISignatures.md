pub struct Logger {
    log_dir: std::path::PathBuf,
}

impl Logger {
    pub fn new(suffix: &str) -> Result<Self, crate::app_error::AppError>;
    pub fn new_with_root(root: &std::path::Path, suffix: &str) -> Result<Self, crate::app_error::AppError>;
    pub fn log_text(&self, file_name: &str, content: &str) -> Result<(), crate::app_error::AppError>;
    pub fn log_json(&self, file_name: &str, content: &serde_json::Value) -> Result<(), crate::app_error::AppError>;
}