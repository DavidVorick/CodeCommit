# User Specification

This is a specification for a tool that uses agentic workflows to assist with
programming tasks. This tool offers multiple agentic flows, each with its own
objective. All workflows are implemented by the 'code-commit' binary.

## Project Design

The CodeCommit project is itself built with code-commit, and follows
code-commit best practices. This means that there is one high level
UserSpecification which outlines the major functions of the project, as well as
smaller modules that each have their own UserSpecification. All of the
UserSpecification documents work together to define the project.

## Supported Workflows

The code-commit binary supports multiple workflows, each of which can be
triggered with a command line flag. It is an error to trigger more than one
workflow at a time.

### Committing Code

The 'committing-code' workflow uses LLMs to run programming tasks and is the
default workflow of the 'code-commit' binary. Other workflows can be specified
with flags, and if no workflow flags are provided the binary will assume that
it is supposed to execute the workflow for committing code.

The programmatic slug that refers to this workflow is 'committing-code'.

The committing code workflow has a 'refactor' modification which can be
triggered by the flags '--refactor' or '--ref'. If either of these flags are
used, the [initial query system prompt] will be replaced by the [refactor query
system prompt].

The user must provide guidance for the committing-code workflow by writing
their own query in agent-config/query.txt

### Checking Consistency

The 'consistency' workflow uses LLMs to verify that the project is self
consistent. It does not write any code, but rather produces output for the user
that tells them about the state of their project.

More specifically, it looks for places where the UserSpecification is
inconsistent with itself, and it looks for places where the UserSpecification
is inconsistent with the current implementation. It then compiles a report.

The report may either be read by a user or by another agentic workflow,
therefore it must be both human readable and machine readable.

The consistency workflow can be triggered with the command line flag
'--consistency-check' or '--consistency' or '--cc'. The programmatic slug that
refers to this workflow is 'consistency'.

The user can optionally provide guidance for the consistency workflow by
writing their own query in agent-config/consistency-query.txt - if none is
provided, an empty string will be used.

## LLMs

CodeCommit supports multiple LLMs. The default LLM is gemini-2.5-pro, and other
LLMs are also supported. To run a different model, the user should pass a
'--model' flag. Unrecognized models and unrecognized flags should produce an
error.

The core logic for interfacing with llms is in the 'llm' module.

### Gemini 2.5 Pro

User flag: '--model gemini-2.5-pro'

API Key Location: agent-config/gemini-key.txt

### GPT 5

User flag: '--model gpt-5'

API Key Location: agent-config/openai-key.txt

## Logging

When any of the agentic workflows are running, they will be logging their
activity in the agent-config/logs/ directory.

The core logic for interfacing with the logs is in the 'logging' module.
