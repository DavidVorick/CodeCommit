use crate::error::AppError;
use anyhow::{Context, Result};
use tokio::fs;

#[derive(Debug, Clone)]
pub struct Config {
    pub api_key: String,
    pub system_prompt: String,
    pub basic_query: String,
    pub codebase_context: String,
}

impl Config {
    async fn load_file(path: &str) -> Result<String> {
        fs::read_to_string(path)
            .await
            .with_context(|| format!("Failed to read file: {}", path))
    }

    pub async fn load() -> Result<Self> {
        let api_key = Self::load_file("gemini-key.txt").await?.trim().to_string();
        if api_key.is_empty() {
            return Err(AppError::Config("gemini-key.txt is empty".to_string()).into());
        }

        let system_prompt = Self::load_file("gemini-system-prompt.txt").await?;
        let basic_query = Self::load_file("query.txt").await?;
        let codebase_context = Self::load_file("context.txt").await?;

        Ok(Self {
            api_key,
            system_prompt,
            basic_query,
            codebase_context,
        })
    }
}