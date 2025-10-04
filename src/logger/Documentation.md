# logger

The logger module handles creating and writing to log files for each workflow run.

## `struct Logger`

Manages logging for a workflow run. Creates a timestamped directory in `agent-config/logs/` on creation.

### `Logger::new(suffix: &str) -> Result<Self, AppError>`

Creates a new logger. `suffix` is appended to the log directory name, e.g., "committing-code".

### Methods

- `log_query_text(&self, prefix: &str, content: &str) -> Result<(), AppError>`
  - Logs to `<prefix>-query.txt`

- `log_query_json(&self, prefix: &str, content: &Value) -> Result<(), AppError>`
  - Logs to `<prefix>-query.json`

- `log_response_json(&self, prefix: &str, content: &Value) -> Result<(), AppError>`
  - Logs to `<prefix>-response.json`

- `log_response_text(&self, prefix: &str, content: &str) -> Result<(), AppError>`
  - Logs to `<prefix>-response.txt`

- `log_build_output(&self, prefix: &str, content: &str) -> Result<(), AppError>`
  - Logs to `<prefix>-build.txt`

- `log_final_error(&self, error: &AppError) -> Result<(), std::io::Error>`
  - Logs to `final_error.txt`

## `mod llm_caller`

Provides a convenient wrapper for calling an LLM and logging the interaction.

### `call_llm_and_log(llm_client: &LlmApiClient, request_body: &Value, logger: &Logger, log_prefix: &str) -> Result<String, AppError>`

Calls an LLM, measures response time, and logs the request and response using the provided logger. Returns the extracted text from the LLM response on success.