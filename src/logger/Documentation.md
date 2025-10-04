# Logger Module Documentation

This document provides a comprehensive guide to the `logger` module's exports and APIs.

## Overview

The `logger` module is responsible for creating and managing log files for each run of an agentic workflow. It ensures that all relevant information, from LLM queries and responses to build outputs and final errors, is stored in a structured and easily accessible manner.

When a workflow starts, a new `Logger` instance is created. This instance automatically creates a unique, timestamped directory inside `agent-config/logs/`. All subsequent logging calls on that instance will write files into this specific directory.

## Structs

### `Logger`

The primary struct that manages logging for a workflow run.

#### Creation

##### `Logger::new(suffix: &str) -> Result<Self, AppError>`

Creates a new `Logger` instance. This will create a new directory for logging.

-   **Arguments:**
    -   `suffix: &str`: A string to append to the log directory name. This is typically the name of the workflow being run (e.g., "committing-code").
-   **Returns:**
    -   `Ok(Logger)`: A new logger instance on success.
    -   `Err(AppError)`: An I/O error if the log directory cannot be created.
-   **Directory Naming:** The created directory will be named `agent-config/logs/YYYY-MM-DD-HH-MM-SS-<suffix>`.

#### Methods

All logging methods write files into the directory created when the `Logger` was instantiated.

##### `log_query_text(&self, prefix: &str, content: &str) -> Result<(), AppError>`

Logs the raw text of a query sent to an LLM.

-   **Arguments:**
    -   `prefix: &str`: A prefix for the filename, typically indicating the attempt number and purpose (e.g., "1-initial-query").
    -   `content: &str`: The query text to log.
-   **Filename:** `<prefix>-query.txt`

##### `log_query_json(&self, prefix: &str, content: &Value) -> Result<(), AppError>`

Logs the full JSON request body sent to an LLM.

-   **Arguments:**
    -   `prefix: &str`: A prefix for the filename.
    -   `content: &Value`: The `serde_json::Value` representing the request body.
-   **Filename:** `<prefix>-query.json`

##### `log_response_json(&self, prefix: &str, content: &Value) -> Result<(), AppError>`

Logs the full JSON response received from an LLM.

-   **Arguments:**
    -   `prefix: &str`: A prefix for the filename.
    -   `content: &Value`: The `serde_json::Value` representing the response body.
-   **Filename:** `<prefix>-response.json`

##### `log_response_text(&self, prefix: &str, content: &str) -> Result<(), AppError>`

Logs the extracted text content from an LLM's response.

-   **Arguments:**
    -   `prefix: &str`: A prefix for the filename.
    -   `content: &str`: The response text to log.
-   **Filename:** `<prefix>-response.txt`

##### `log_build_output(&self, prefix: &str, content: &str) -> Result<(), AppError>`

Logs the output (stdout and stderr) of a build script run.

-   **Arguments:**
    -   `prefix: &str`: A prefix for the filename, used to associate the build with a specific LLM call.
    -   `content: &str`: The build output to log.
-   **Filename:** `<prefix>-build.txt`

##### `log_final_error(&self, error: &AppError) -> Result<(), std::io::Error>`

Logs the final error that caused the workflow to terminate. This is intended to be called just before the program exits with an error.

-   **Arguments:**
    -   `error: &AppError`: The application error to log.
-   **Filename:** `final_error.txt`