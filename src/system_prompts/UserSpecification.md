dependencies:

# System Prompts

The system_prompts module contains constants which define different components
of a system prompt that can be passed into an LLM. Most agentic workflows use
some combination of system prompt constants from this module when composing
their own system prompts.

The system prompts are all text files that are created by the user. The LLM is
to update mod.rs so that it contains constants which load the files at
compilation and share a name with the filename of the .txt file. Every .txt
file in this folder should have a corresponding constant in mod.rs

## Available Prompts

- `CODE_MODIFICATION_INSTRUCTIONS: &str`
- `CONTEXT_BUILDER_CONTEXT_QUERY: &str`
- `COMMITTING_CODE_INITIAL_QUERY: &str`
- `COMMITTING_CODE_REPAIR_QUERY: &str`
- `COMMITTING_CODE_EXTRA_CODE_QUERY: &str`
- `CONSISTENCY_CHECK: &str`
- `PROJECT_STRUCTURE: &str`
