use crate::app_error::AppError;
use chrono::Utc;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

pub struct Logger {
    log_dir: PathBuf,
}

impl Logger {
    pub fn new() -> Result<Self, AppError> {
        let timestamp = Utc::now().format("%Y-%m-%d_%H-%M-%S").to_string();
        let log_dir = PathBuf::from("logs").join(timestamp);
        fs::create_dir_all(&log_dir)?;
        Ok(Self { log_dir })
    }

    fn path_for(&self, file_name: &str) -> PathBuf {
        self.log_dir.join(file_name)
    }

    pub fn log_prompt(&self, attempt: u32, content: &str) -> Result<(), AppError> {
        let path = self.path_for(&format!("query-{attempt}.txt"));
        fs::write(path, content)?;
        Ok(())
    }

    pub fn log_response_json(&self, attempt: u32, content: &Value) -> Result<(), AppError> {
        let path = self.path_for(&format!("query-{attempt}-response.json"));
        let pretty_json = serde_json::to_string_pretty(content)?;
        fs::write(path, pretty_json)?;
        Ok(())
    }

    pub fn log_response_text(&self, attempt: u32, content: &str) -> Result<(), AppError> {
        let path = self.path_for(&format!("query-{attempt}-response.txt"));
        fs::write(path, content)?;
        Ok(())
    }

    pub fn log_build_output(&self, attempt: u32, content: &str) -> Result<(), AppError> {
        let path = self.path_for(&format!("query-{attempt}-build.txt"));
        fs::write(path, content)?;
        Ok(())
    }
}
