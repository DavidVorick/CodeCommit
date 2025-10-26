# `committing_code` Module Internal Dependencies

This document lists all APIs that the `committing_code` module uses from other modules within this project.

## `llm` module

- `llm::query(model: Model, api_key: String, prompt: &str, logger: &Logger, log_prefix: &str) -> Result<String, AppError>`

## `logger` module

- `logger::Logger::log_text(&self, file_name: &str, content: &str) -> Result<(), AppError>`

## `system_prompts` module

- `system_prompts::CODE_MODIFICATION_INSTRUCTIONS`
- `system_prompts::COMMITTING_CODE_INITIAL_QUERY`
- `system_prompts::COMMITTING_CODE_REFACTOR_QUERY`
- `system_prompts::COMMITTING_CODE_REPAIR_QUERY`
- `system_prompts::PROJECT_STRUCTURE`