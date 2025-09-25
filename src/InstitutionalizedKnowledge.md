Based on a review of the implementation, the following non-obvious knowledge has been identified. This knowledge was likely acquired through the process of building, deploying, and refining the tool, and is not explicitly captured in the User Specification.

### Deterministic Prompts for LLM Repair Attempts

When constructing a repair prompt for a failed build, the list of file replacements that were previously applied is included in the prompt. The implementation sorts these files alphabetically by path before adding them to the prompt.

This practice is crucial for ensuring that the generated prompts are deterministic. Non-deterministic prompts, where the order of file replacements could change between runs, can lead to non-reproducible LLM outputs and make debugging agentic behavior significantly more difficult. Enforcing a consistent order is a key lesson in managing complex LLM interactions.

### Robust Build Script Test Execution

The `build.sh` script is designed to be robust against projects that may not have a full suite of tests. Specifically, it uses `cargo test -- --list` combined with shell commands like `comm` to determine if any regular or ignored tests exist *before* attempting to run them with `cargo nextest run`.

This prevents the build from failing due to the test runner exiting with an error when it's asked to run a category of tests that doesn't exist. This is a practical consideration for making the build script resilient across different project states.

### Flexible `.gitignore` Safety Checks

The specification requires verifying that API keys in the `agent-config` directory are not committed to git. The implementation of this check is more flexible than a simple check for one specific line. It first looks for a broad rule that ignores the entire directory (e.g., `/agent-config` or `agent-config/`). If and only if such a rule is not found, it proceeds to look for more specific rules that ignore the individual key files (e.g., `/agent-config/gemini-key.txt`).

This two-tiered approach provides a better user experience, allowing users to ignore the directory in the most common way without being forced to add multiple, more specific rules.

### Final Error Logging for Debugging

In addition to the standard logging of LLM calls and build attempts, the application is configured to perform a final logging action if the main workflow terminates with an error. The text of the final, unhandled `AppError` is written to `final_error.txt` within the log directory for that run.

This ensures that even in cases of unexpected or catastrophic failure (e.g., a configuration error that prevents any LLM calls), a clear record of the root cause is preserved, which is invaluable for debugging the agentic system.

### Lenient Parsing of LLM Responses

The parser for the `^^^file...^^^end` syntax is intentionally lenient. If a file block is opened with `^^^path/to/file` but is not properly closed with `^^^end` (e.g., because the LLM's output was truncated), the parser will treat all subsequent text until the end of the response as the content for that file.

This design choice prevents the entire workflow from failing due to a minor formatting mistake by the LLM and attempts to salvage what might otherwise be a valid and useful code modification.

### Code Rollup Exclusions for `assets` Directories

The `codeRollup.sh` script, which aggregates source code for the LLM context, is hardcoded to skip any directory named `assets`, regardless of its location in the project structure. This is a practical heuristic to prevent large binary files, images, fonts, and other non-textual assets from being wastefully included in the `codeRollup.txt` file, which would consume expensive tokens and potentially confuse the LLM.

### Nuances of `.gitignore` Parsing for Path Protection

The implementation and its tests reveal that correctly respecting `.gitignore` rules for path protection is subtle. A key lesson learned is the importance of anchoring paths. For example, a rule like `/target/` is used instead of `target/` to ensure that only the project's root `target` directory is ignored, preventing the accidental exclusion of valid source code files in a directory like `src/target/`. This level of precision is necessary to prevent the agent from being blocked from modifying legitimate files.