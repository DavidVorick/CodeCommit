# `context_builder` Module Internal Dependencies

This document lists all APIs that the `context_builder` module uses from other modules within this project.

## `config` module

- Uses the `Config` struct.

## `llm` module

- `llm::query(model: Model, api_key: String, prompt: &str, logger: &Logger, log_prefix: &str) -> Result<String, AppError>`

## `logger` module

- `logger::Logger` struct is used.

## `system_prompts` module

- `system_prompts::CONTEXT_BUILDER_CONTEXT_QUERY`
- `system_prompts::PROJECT_STRUCTURE`