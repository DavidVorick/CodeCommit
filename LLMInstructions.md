# LLM Instructions

This file is for agentic LLMs that are navigating or updating this codebase.
LLMs should follow all of the instructions provided in this file.

## Testing

Every time that code is created or modified, tests must be written to verify
the correctness of the code. Six types of tests must be written for every
function:

1. Unit tests. Each function needs to be unit tested.
2. Happy path testing. The expected use case of every API must be tested.
3. Branch testing. Tests must be written to use every branch in the codebase.
4. Error testing. Tests must verify that every error is handled gracefully.
5. Edge case testing. Tests must be written to comprehensively cover every edge
   case.
6. Adversarial testing. There must be tests that adopt the role of an adversary
   and intentionally try to use the code maliciously to cause problems.

Testing is broken into two categories: short testing and long testing. The
short test suite must always finish within 20 seconds, and the long test suite
must always finish within 90 seconds.

The build uses `cargo clippy -- -D warnings` as part of the build step. This
means that the build will fail if there are any unused imports, or if there are
println! statements that aren't using inline arguments, if variables aren't
named in snake_case, and so on. Code must be written to a standard that will
satisfy the strict approach to clippy's warnings.
