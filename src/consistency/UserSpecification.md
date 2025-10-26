# Checking Consistency

This is a specification for the implementation of an agentic coding workflow
that builds context for an LLM that reviews code and then generates a report.
The report is placed as a standalone file in
agent-config/consistency-report.txt.

## The Query

The binary builds context for an LLM that requests code review, and then it
calls the LLM to get a response. The prompt will have the following format:

[project structure system prompt]
[consistency check system prompt]
[query]
[codebase]

The query is then sent to the LLM, and the text response is recorded in
agent-config/consistency-report.txt.
