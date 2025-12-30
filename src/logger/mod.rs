use crate::app_error::AppError;
use chrono::Utc;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

#[cfg(test)]
mod logger_test;

pub struct Logger {
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

    pub fn log_text(&self, file_name: &str, content: &str) -> Result<(), AppError> {
        let path = self.path_for(file_name);
        fs::write(path, content)?;
        Ok(())
    }

    pub fn log_json(&self, file_name: &str, content: &Value) -> Result<(), AppError> {
        let path = self.path_for(file_name);
        let pretty_json = serde_json::to_string_pretty(content)?;
        fs::write(path, pretty_json)?;
        Ok(())
    }
}
