# `src` Internal Dependencies

This document lists all APIs that the top-level `src` code (e.g., `main.rs`, `config.rs`) uses from other modules within this project.

## `committing_code` module

- `committing_code::run(logger: &logger::Logger, cli_args: CliArgs) -> Result<(), AppError>`

## `consistency` module

- `consistency::run(logger: &logger::Logger, cli_args: CliArgs) -> Result<(), AppError>`

## `logger` module

- `logger::Logger::new(suffix: &str) -> Result<Self, AppError>`
- `logger::Logger::log_text(&self, file_name: &str, content: &str) -> Result<(), AppError>`

## `init` module

- `init::run_init_command(project_name: &str) -> Result<(), AppError>`

## `rollup` module

- `rollup::run(logger: &logger::Logger, cli_args: CliArgs) -> Result<(), AppError>`

## `auto_workflow` module

- `auto_workflow::run(logger: &logger::Logger, cli_args: CliArgs) -> Result<(), AppError>`