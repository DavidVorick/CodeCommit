# `llm` Module Internal Dependencies

This document lists all APIs that the `llm` module uses from other modules within this project.

## `logger` module

- `logger::Logger::log_response_json(&self, prefix: &str, content: &Value) -> Result<(), AppError>`
- `logger::Logger::log_response_text(&self, prefix: &str, content: &str) -> Result<(), AppError>`