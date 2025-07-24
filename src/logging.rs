use anyhow::{Context, Result};
use chrono::Local;
use std::path::{Path, PathBuf};
use tokio::fs::{self, OpenOptions};
use tokio::io::AsyncWriteExt;

pub struct Logger {
    log_dir: PathBuf,
}

impl Logger {
    pub fn new() -> Result<Self> {
        let timestamp = Local::now().format("%Y%m%d-%H%M%S").to_string();
        let log_dir = PathBuf::from("logs").join(timestamp);
        std::fs::create_dir_all(&log_dir)
            .with_context(|| format!("Failed to create log directory: {}", log_dir.display()))?;
        Ok(Self { log_dir })
    }

    pub fn log_dir(&self) -> &Path {
        &self.log_dir
    }

    pub async fn log_prompt(&self, number: usize, content: &str) -> Result<()> {
        let path = self.log_dir.join(format!("query-{}.txt", number));
        fs::write(&path, content).await?;
        Ok(())
    }

    pub async fn log_llm_response(
        &self,
        number: usize,
        full_response: &str,
        text_content: &str,
    ) -> Result<()> {
        let json_path = self.log_dir.join(format!("query-{}-response.json", number));
        fs::write(&json_path, full_response).await?;

        let txt_path = self.log_dir.join(format!("query-{}-response.txt", number));
        fs::write(&txt_path, text_content).await?;
        Ok(())
    }

    pub async fn log_build_output(&self, number: usize, content: &str) -> Result<()> {
        let path = self.log_dir.join(format!("query-{}-build.txt", number));
        fs::write(&path, content).await?;
        Ok(())
    }

    pub async fn log_user_thought(&self, thought: &str) -> Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open("llm-user-output.txt")
            .await?;

        let formatted_thought = format!("{}\n---\n", thought);
        file.write_all(formatted_thought.as_bytes()).await?;
        Ok(())
    }
}
