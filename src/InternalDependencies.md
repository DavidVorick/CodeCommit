# `src` Internal Dependencies

This document lists all APIs that the top-level `src` code (e.g., `main.rs`, `config.rs`) uses from other modules within this project.

## `llm` module

- `llm::api::GeminiClient::new(api_key: String) -> Self`
- `llm::api::GptClient::new(api_key: String) -> Self`
- `llm::api::LlmApiClient` (enum)
- `llm::api::LlmApiClient::get_url(&self) -> &'static str`
- `llm::api::LlmApiClient::build_request_body(&self, prompt: &str) -> Value`
- `llm::caller::call_llm_and_log(...) -> Result<String, AppError>`

## `logger` module

- `logger::Logger::new(suffix: &str) -> Result<Self, AppError>`
- `logger::Logger::log_query_text(&self, prefix: &str, content: &str) -> Result<(), AppError>`
- `logger::Logger::log_query_json(&self, prefix: &str, content: &Value) -> Result<(), AppError>`
- `logger::Logger::log_response_json(&self, prefix: &str, content: &Value) -> Result<(), AppError>`
- `logger::Logger::log_response_text(&self, prefix: &str, content: &str) -> Result<(), AppError>`
- `logger::Logger::log_build_output(&self, prefix: &str, content: &str) -> Result<(), AppError>`
- `logger::Logger::log_final_error(&self, error: &AppError) -> Result<(), std::io::Error>`