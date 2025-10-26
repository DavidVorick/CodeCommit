# `logger` Module Public API

This document enumerates the full public API of the `logger` module.

## `struct Logger`
- `new(suffix: &str) -> Result<Self, AppError>`
- `log_text(&self, file_name: &str, content: &str) -> Result<(), AppError>`
- `log_json(&self, file_name: &str, content: &Value) -> Result<(), AppError>`