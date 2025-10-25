# LLM Module

This module provides clients and helpers for interacting with Large Language Model APIs.

## `mod api`

Handles the direct communication with LLM APIs.

### `struct GeminiClient`
A client for the Gemini API.
- `new(api_key: String) -> Self`
- `query(&self, request_body: &Value) -> Result<Value, AppError>`

### `struct GptClient`
A client for the GPT API.
- `new(api_key: String) -> Self`
- `query(&self, request_body: &Value) -> Result<Value, AppError>`

### `enum LlmApiClient`
An enum that abstracts over different LLM clients.
- `Gemini(GeminiClient)`
- `Gpt(GptClient)`

#### Methods
- `get_url(&self) -> &'static str`
- `build_request_body(&self, prompt: &str) -> Value`
- `query(&self, request_body: &Value) -> Result<Value, AppError>`
- `extract_text_from_response(&self, response: &Value) -> Result<String, AppError>`

### Free Functions
- `extract_text_from_gemini_response(response: &Value) -> Result<String, AppError>`
- `extract_text_from_gpt_response(response: &Value) -> Result<String, AppError>`

## `mod caller`

Provides a high-level function to call an LLM and handle logging.

### `call_llm_and_log(llm_client: &LlmApiClient, request_body: &Value, logger: &Logger, log_prefix: &str) -> Result<String, AppError>`
Calls an LLM, measures response time, and logs the request and response using the provided logger. Returns the extracted text from the LLM response on success.