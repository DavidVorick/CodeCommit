# Refactor Module Documentation

This document provides a guide to the `refactor` module's exports.

## Overview

The `refactor` module contains system prompts used specifically for the code refactoring workflow.

## Exports

### `prompts` module

This module contains the string constants for system prompts.

-   `REFACTOR_INITIAL_QUERY_SYSTEM_PROMPT: &str`: The system prompt for the initial refactoring request to the LLM.
-   `REFACTOR_REPAIR_QUERY_SYSTEM_PROMPT: &str`: The system prompt for a repair request to the LLM when a refactoring attempt fails to build.