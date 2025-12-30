# `committing_code` Internal Dependencies

This document lists all APIs that the `committing_code` module uses from other modules within this project.

## `cli` module

- `cli::CliArgs`

## `config` module

- `config::Config`
- `config::Config::load(args: &CliArgs) -> Result<Self, AppError>`

## `context_builder` module

- `context_builder::build_codebase_context(next_agent_full_prompt: &str, config: &config::Config, logger: &logger::Logger) -> Result<String, AppError>`

## `llm` module

- `llm::query(model: Model, api_key: String, prompt: &str, logger: &Logger, log_prefix: &str) -> Result<String, AppError>`

## `logger` module

- `logger::Logger`
- `logger::Logger::log_text(&self, file_name: &str, content: &str) -> Result<(), AppError>`

## `system_prompts` module

- `system_prompts::CODE_MODIFICATION_INSTRUCTIONS`
- `system_prompts::COMMITTING_CODE_EXTRA_CODE_QUERY`
- `system_prompts::COMMITTING_CODE_INITIAL_QUERY`
- `system_prompts::COMMITTING_CODE_REPAIR_QUERY`
- `system_prompts::PROJECT_STRUCTURE`