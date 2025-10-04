# Project Modules

This document enumerates and describes the modules that make up the CodeCommit project.

## `app_error`
-   **Description**: Defines the project's custom error types, `AppError` and `BuildFailure`.
-   **Dependencies**: None.

## `build_runner`
-   **Description**: Provides a function to execute the `build.sh` script and capture its output.
-   **Dependencies**: `app_error`.

## `cli`
-   **Description**: Handles parsing of command-line arguments to determine the requested workflow and model.
-   **Dependencies**: `app_error`.

## `config`
-   **Description**: Manages configuration by loading API keys and prompts from files, processing CLI arguments, and constructing prompts for the LLM.
-   **Dependencies**: `app_error`, `cli`, `prompts`, `prompts_consistency`, `refactor`.

## `file_updater`
-   **Description**: Applies file modifications (create, update, delete) based on the parsed LLM response. Includes path validation to protect critical files and directories.
-   **Dependencies**: `app_error`, `response_parser`.

## `llm_api`
-   **Description**: Provides clients for interacting with different LLM APIs (Gemini and GPT). It handles the specifics of building requests and parsing responses for each API.
-   **Dependencies**: `app_error`.

## `logger`
-   **Description**: Provides logging functionality for agentic workflows. Creates a timestamped directory for each run and writes logs for LLM queries, responses, build outputs, and errors. It includes a helper function for making LLM calls that automatically logs the request and response.
-   **Dependencies**: `app_error`, `llm_api`.

## `prompts`
-   **Description**: Contains system prompt constants for the 'committing-code' workflow.
-   **Dependencies**: None.

## `prompts_consistency`
-   **Description**: Contains the system prompt constant for the 'consistency' workflow.
-   **Dependencies**: None.

## `refactor`
-   **Description**: Contains system prompts used by the 'refactor' workflow. This workflow uses LLMs to restructure code to be compliant with agentic workflow standards.
-   **Dependencies**: None.

## `response_parser`
-   **Description**: Parses the text response from an LLM to extract file update instructions.
-   **Dependencies**: `app_error`.