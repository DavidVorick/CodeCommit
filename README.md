# CodeCommit

CodeCommit is a rust-based CLI tool that manages an automated workflow for
using LLMs to create new code commits for rust projects. It automatically runs
a loop that asks an LLM to write code based on a provided query, then it
updates the codebase and runs the build. If the build fails, it feeds the build
output back into the LLM to get any issues fixed, and then tries again.

---

## Quick start

This is a standard rust project that should build with 'cargo build'. The full
build script is:

```
cargo fmt
cargo build
cargo nextest run
cargo nextest run -- --ignored
cargo clippy -- -D warnings
```

### Configuration

To run this tool in another project, you will need the following text files:

| File | Purpose |
|------|---------|
| `gemini-key.txt` | A Gemini API Key |
| `project-prompt.txt` | A project specific prompt to be included in all queries |
| `query.txt` | The specific query for the current code commit |
| `codeRollup.txt` | The rolled up codebase |
| `build.sh` | The build script for the project |

### Execution

The binary takes the following steps:

1. Build an initial prompt from the config files
2. Call Gemini and parse the response
3. Apply any file replacements provided by Gemini
4. Run the build
5. If the build fails, ask Gemini to fix the build, repeating a small number of times.

Logs are written to `logs/<TIMESTAMP>/`.

## LLM response syntax

The LLM is instructed to follow very strict syntax in its response. If the LLM
response does not perfectly match the requested formatting, it is treated as an
error and will cause the binary to exit.

If the LLM wishes to replace a file in the codebase, it must use the following
syntax:

```
^^^<filepath>
// example file contents
^^^end
```

## Security notes

* Paths are normalised with [`path-clean`](https://docs.rs/path-clean) and **must not**
  contain `..`, absolute roots, or reference `.git`.
* LLM API errors, malformed blocks, or filesystem I/O issues surface as `AppError`
* All outbound traffic is restricted to a single HTTPS request per iteration.
