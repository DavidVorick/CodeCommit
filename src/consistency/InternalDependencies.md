# `consistency` Module Internal Dependencies

This document lists all APIs that the `consistency` module uses from other modules within this project.

## `cli` module

- Uses the `CliArgs` struct as input to the workflow.

## `config` module

- `config::Config::load(args: &CliArgs) -> Result<Config, AppError>`

## `context_builder` module

- `context_builder::build_codebase_context(next_agent_full_prompt: &str, config: &config::Config, logger: &logger::Logger) -> Result<String, AppError>`

## `llm` module

- `llm::query(model: Model, api_key: String, prompt: &str, logger: &Logger, log_prefix: &str) -> Result<String, AppError>`

## `logger` module

- `logger::Logger::log_text(&self, file_name: &str, content: &str) -> Result<(), AppError>`

## `system_prompts` module

- `system_prompts::CONSISTENCY_CHECK`
- `system_prompts::PROJECT_STRUCTURE`