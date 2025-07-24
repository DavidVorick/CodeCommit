# CodeCommit

CodeCommit is a rust-based CLI tool that automates using LLMs to create new
code commits for projects. It automatically runs a loop for developers that
asks an LLM to write code, then builds the code, then feeds the output of the
build script back into the LLM to have any issues repaired.

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

To use this tool in another repository, you will need a gemini API key as well.

### Configuration

To run this tool in another project, you will need the following text files:

| File | Purpose |
|------|---------|
| `gemini-key.txt` | One‑line Google Generative Language API key |
| `gemini-system-prompt.txt` | System prompt sent to Gemini |
| `query.txt` | “Basic query” framing every request |
| `context.txt` | Codebase context (e.g. output of `codeRollup.sh`) |

### Execution

After running the binary, it will:

1. Builds the initial prompt from the four config files.  
2. Calls Gemini and **parses the response** (see syntax below).  
3. Applies file replacements, then runs `./build.sh`.  
4. On failure, feeds the compiler output back to Gemini and repeats.
5. It will ask the LLM to repair the code up to 8 times total.

Logs are written to `logs/<TIMESTAMP>/`.

## LLM response syntax

All LLM outputs **must** use the following non‑overlapping blocks:

| Marker | Usage | Parsed action |
|--------|-------|---------------|
| `&&&start … &&&end` | Narration for the human operator | Printed to `stdout` and appended to `llm-user-output.txt` |
| `%%%start … %%%end` | Internal debug thoughts | Held in memory, embedded in next prompt |
| `^^^<path> … ^^^end` | Full replacement of a file | Safely written to disk (no path traversal, `.git` forbidden) |
| `$$$start … $$$end` | Signal that **no further changes** are required | Terminates the loop if the build already passes |

Violations (overlap, duplicate files, missing end tags, both `^^^` and `$$$`,
etc.) are treated as build failures and included verbatim in the next prompt.

## Security notes

* Paths are normalised with [`path-clean`](https://docs.rs/path-clean) and **must not**
  contain `..`, absolute roots, or reference `.git`.
* LLM API errors, malformed blocks, or filesystem I/O issues surface as
  `AppError` and break the current iteration.
* All outbound traffic is restricted to a single HTTPS request per iteration.
