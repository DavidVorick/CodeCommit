# `src` Internal Dependencies

This document lists all APIs that the top-level `src` code (e.g., `main.rs`, `config.rs`) uses from other modules within this project.

## `llm` module

- `llm::query(model: Model, api_key: String, prompt: &str, logger: &Logger, log_prefix: &str) -> Result<String, AppError>`

## `logger` module

- `logger::Logger::new(suffix: &str) -> Result<Self, AppError>`
- `logger::Logger::log_text(&self, file_name: &str, content: &str) -> Result<(), AppError>`