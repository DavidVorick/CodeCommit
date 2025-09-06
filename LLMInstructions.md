# LLM Instructions

This file is for agentic LLMs that are navigating or updating this codebase.
LLMs should follow all of the instructions provided in this file.

## Code Quality Instructions

This codebase attempts to be a professional codebase with high code quality
standards. This means using idiomatic patterns, handling every error case, and
generating robust testing for each piece of code. Code must be developed with
security and adversarial conditions in mind. Code should also be plain and
simple - any junior engineer should be able to follow along with the code even
if there aren't many comments.

When coding, you are meticulous, professional, and demonstrate considerable
skill and experience in completing your task as instructed.

## Code File Sizes

Within reason, code files are not to exceed 250 lines of code. If a code file
is starting to approach that size, it should be split into multiple smaller
code files.

Tests must always be kept in their own files. Do not mix production logic with
testing logic. Tests must also adhere to the policy of staying below 250 lines
of code per file.

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

## Comments

Code should be kept as clean and as minimal as possible, which also means that
there should be minimal comments. The vast majority of the code will only ever
be viewed by LLMs, which means that comments are only necessary if it will be
helpful to an LLM. The majority of code will not need any comments at all.

## Documentation

The documentation will only ever be read by LLMs. This means that documentation
does not need to be pretty, and instead should optimize for presenting
information clearly without consuming too many tokens.

## Rust Conventions

The build will always run the following commands:

```
cargo fmt
cargo build
cargo nextest run
cargo nextest run -- --ignored
cargo clippy -- -D warnings
```

All five commands need to pass without error for a code update to be considered
successful. If a code update is not successful, the output will be thrown away.

All common rust idioms and best practices must be followed.

All files should be named using `snake_case`. All tests should have the
`_test.rs` suffix.

All custom error types should implement the std::fmt::Display trait as well as
the std::error::Error trait.
