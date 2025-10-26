# `consistency` Module Internal Dependencies

This document lists all APIs that the `consistency` module uses from other modules within this project.

## `cli` module

- Uses the `CliArgs` struct as input to the workflow.

## `config` module

- `config::Config::load(args: &CliArgs) -> Result<Config, AppError>`

## `llm` module

- `llm::query(model: Model, api_key: String, prompt: &str, logger: &Logger, log_prefix: &str) -> Result<String, AppError>`

## `logger` module

The `logger` module's `Logger` struct is passed to other modules, but no `Logger` methods are directly called.

## `system_prompts` module

- `system_prompts::CONSISTENCY_CHECK`
- `system_prompts::PROJECT_STRUCTURE`