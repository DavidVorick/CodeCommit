# `logger` Module Public API

This document enumerates the full public API of the `logger` module.

## `struct Logger`
- `new(suffix: &str) -> Result<Self, AppError>`
- `log_query_text(&self, prefix: &str, content: &str) -> Result<(), AppError>`
- `log_query_json(&self, prefix: &str, content: &Value) -> Result<(), AppError>`
- `log_response_json(&self, prefix: &str, content: &Value) -> Result<(), AppError>`
- `log_response_text(&self, prefix: &str, content: &str) -> Result<(), AppError>`
- `log_build_output(&self, prefix: &str, content: &str) -> Result<(), AppError>`
- `log_final_error(&self, error: &AppError) -> Result<(), std::io::Error>`