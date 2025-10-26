# `system_prompts` Module Public API

This module exports string constants containing system prompts for various agentic workflows.

## Constants

- `CODE_MODIFICATION_INSTRUCTIONS: &str`: Instructions for the LLM on how to format code modifications.
- `COMMITTING_CODE_INITIAL_QUERY: &str`: The system prompt for the initial query in the 'committing code' workflow.
- `COMMITTING_CODE_REPAIR_QUERY: &str`: The system prompt for a repair query in the 'committing code' workflow.
- `CONSISTENCY_CHECK: &str`: The system prompt for the 'consistency check' workflow.
- `PROJECT_STRUCTURE: &str`: A description of the CodeCommit project structure to be included in prompts.
- `REFACTOR: &str`: The system prompt for the initial query in the 'refactor' workflow.