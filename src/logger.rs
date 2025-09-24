use crate::app_error::AppError;
use chrono::Utc;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

pub struct Logger {
    // This field remains private, but we expose its path via a public method.
    log_dir: PathBuf,
}

impl Logger {
    pub fn new(suffix: &str) -> Result<Self, AppError> {
        let timestamp = Utc::now().format("%Y-%m-%d-%H-%M-%S").to_string();
        let dir_name = if suffix.is_empty() {
            timestamp
        } else {
            format!("{timestamp}-{suffix}")
        };
        let log_dir = PathBuf::from("agent-config").join("logs").join(dir_name);
        fs::create_dir_all(&log_dir)?;
        Ok(Self { log_dir })
    }

    fn path_for(&self, file_name: &str) -> PathBuf {
        self.log_dir.join(file_name)
    }

    pub fn log_query_text(&self, prefix: &str, content: &str) -> Result<(), AppError> {
        let path = self.path_for(&format!("{prefix}-query.txt"));
        fs::write(path, content)?;
        Ok(())
    }

    pub fn log_query_json(&self, prefix: &str, content: &Value) -> Result<(), AppError> {
        let path = self.path_for(&format!("{prefix}-query.json"));
        let pretty_json = serde_json::to_string_pretty(content)?;
        fs::write(path, pretty_json)?;
        Ok(())
    }

    pub fn log_response_json(&self, prefix: &str, content: &Value) -> Result<(), AppError> {
        let path = self.path_for(&format!("{prefix}-response.json"));
        let pretty_json = serde_json::to_string_pretty(content)?;
        fs::write(path, pretty_json)?;
        Ok(())
    }

    pub fn log_response_text(&self, prefix: &str, content: &str) -> Result<(), AppError> {
        let path = self.path_for(&format!("{prefix}-response.txt"));
        fs::write(path, content)?;
        Ok(())
    }

    pub fn log_build_output(&self, prefix: &str, content: &str) -> Result<(), AppError> {
        let filename = format!("{prefix}-build.txt");
        let path = self.path_for(&filename);
        fs::write(path, content)?;
        Ok(())
    }

    // New public method to handle writing the final error.
    pub fn log_final_error(&self, error: &AppError) -> Result<(), std::io::Error> {
        let path = self.path_for("final_error.txt");
        fs::write(path, error.to_string())
    }
}
