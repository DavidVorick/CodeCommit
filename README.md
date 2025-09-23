# code-commit

## Overview (outdated)

`code-commit` is an agentic software development tool that automates the process of modifying a codebase based on natural language instructions. It integrates with a Large Language Model (LLM) to interpret a user's request, generate the corresponding code changes, and verify them against a build script. If the initial changes fail to build or pass tests, `code-commit` will automatically attempt to repair the code, iterating until the build is successful or a maximum number of attempts has been reached. This tool is designed to be part of a fully automatic, agentic workflow, streamlining development and enabling rapid iteration.

## How to Use

Using `code-commit` involves setting up a few key files in your project's root directory. The `code-commit` binary is then run from this directory. The process is designed to be simple and scriptable. The `code-commit` repository itself is developed using this very tool, serving as a practical demonstration of its capabilities.

The typical workflow is as follows:
1.  A developer writes a set of instructions for the desired code changes into a file named `query.txt`.
2.  A script, `codeRollup.sh`, is run to gather the relevant source code from the project into a single context file, `codeRollup.txt`.
3.  The `code-commit` binary is executed.
4.  The tool reads the query and the codebase context, sends them to an LLM, and receives proposed file changes in response.
5.  These changes are applied directly to the local filesystem.
6.  The project's `build.sh` script is executed to compile, lint, and test the modified code.
7.  If the build is successful, `code-commit` exits, leaving the modified files ready for developer review. If the build fails, `code-commit` will inform the LLM of the failure and ask for a fix, repeating this cycle up to three times.

## Security Features

Security is a primary consideration in the design of `code-commit`. The tool incorporates several safeguards to protect your project. It maintains a strict list of forbidden files and directories, preventing any modifications to critical components such as `Cargo.lock`, `.gitignore`, configuration files, and sensitive directories like `.git/` and `logs/`. The file updater also robustly defends against path traversal attacks, ensuring that file operations are confined to the project directory.

The most critical security feature, however, is the mandatory user oversight. `code-commit` makes changes to your local files, but it does not commit them to your version control system. After the tool completes its work successfully, the developer is expected to review all modifications using standard tools like `git diff`. This step is essential. It provides a human checkpoint to ensure the LLM-generated code is correct, secure, and aligns with the project's goals before it is permanently integrated into the codebase with a `git commit`.

## Key Files

To integrate `code-commit` into your project, you will need to create and configure the following files in your project's root directory:

*   **`query.txt`**: This plain text file contains the natural language prompt describing the changes you want to make. It should be clear and specific to guide the LLM effectively. For example: "Refactor the `user_repository` module to use a connection pool instead of creating a new database connection for each query."

*   **`gemini-key.txt`**: This file should contain only your Google Gemini API key. It is used by `code-commit` to authenticate with the LLM service. This file is included in the default `.gitignore` to prevent accidental exposure of your API key.

*   **`build.sh`**: This executable shell script (`chmod +x build.sh`) is the gatekeeper for code quality. `code-commit` runs this script to verify that the LLM's changes are valid. A comprehensive `build.sh` should perform formatting, linting, compilation, and run the full test suite. It is critical that this script exits with a status code of `0` on success and any non-zero value on failure.

*   **`codeRollup.sh`**: This script is responsible for creating `codeRollup.txt`. Its purpose is to concatenate all relevant source code files into a single text file, providing the LLM with the necessary context about your project's current state. The `code-commit` repository provides a sample script that can be adapted for your own needs.

*   **`LLMInstructions.md` and `UserSpecification.md`**: These markdown files provide persistent, high-level instructions to the LLM. `LLMInstructions.md` sets the rules for how the LLM should behave and format its output, while `UserSpecification.md` describes the project's architecture, goals, and conventions. These files ensure that the LLM's contributions are consistent with your project's standards.
