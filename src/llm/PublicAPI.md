# `llm` Module Public API

This document enumerates the full public API of the `llm` module.

## `api.rs`

### `struct GeminiClient`
- `new(api_key: String) -> Self`
- `query(&self, request_body: &Value) -> Result<Value, AppError>`

### `struct GptClient`
- `new(api_key: String) -> Self`
- `query(&self, request_body: &Value) -> Result<Value, AppError>`

### `enum LlmApiClient`
- `Gemini(GeminiClient)`
- `Gpt(GptClient)`
- `get_model_name(&self) -> &'static str`
- `get_url(&self) -> &'static str`
- `build_request_body(&self, prompt: &str) -> Value`
- `query(&self, request_body: &Value) -> Result<Value, AppError>`
- `extract_text_from_response(&self, response: &Value) -> Result<String, AppError>`

### Free Functions
- `extract_text_from_gemini_response(response: &Value) -> Result<String, AppError>`
- `extract_text_from_gpt_response(response: &Value) -> Result<String, AppError>`

## `caller.rs`

### Free Functions
- `call_llm_and_log(llm_client: &LlmApiClient, request_body: &Value, logger: &Logger, log_prefix: &str) -> Result<String, AppError>`