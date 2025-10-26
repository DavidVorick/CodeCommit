# `llm` Module Internal Dependencies

This document lists all APIs that the `llm` module uses from other modules within this project.

## `logger` module

- `logger::Logger::log_text(&self, file_name: &str, content: &str) -> Result<(), AppError>`
- `logger::Logger::log_json(&self, file_name: &str, content: &Value) -> Result<(), AppError>`