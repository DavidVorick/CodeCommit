# `auto_workflow` Module Internal Dependencies

This document lists all APIs that the `auto_workflow` module uses from other modules within this project.

## `cli` module

- `cli::CliArgs`

## `config` module

- `config::Config`

## `llm` module

- `llm::query(model: Model, api_key: String, prompt: &str, logger: &Logger, log_prefix: &str) -> Result<String, AppError>`

## `logger` module

- `logger::Logger`