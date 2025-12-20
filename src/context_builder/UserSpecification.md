dependencies:
  src/llm
  src/system_prompts

# Building Context

This is a specification for a building block that can be used in agentic coding
workflows. The building block specifically helps to build context for other
agents that will be performing various coding tasks. The context builder can be
thought of as a "preprocessing LLM" which surveys the project files and
documentation and then builds a "codebase" context which includes only the
files that are likely to be necessary for the next agent to complete its task.

### Preprocessing LLM Call

This LLM will be given a prompt with the following
format:

[context query system prompt]
[next agent full prompt]
[codebase summary]

The context query system prompt can be found in the `system_prompts` module.
The 'next agent full prompt' is provided as input by the caller that is
requesting a custom context.

Between the context query system prompt and the next agent full prompt, the
text '=== Next Agent Full Prompt ===' will appear as its own line. Between the
next agent full prompt and the codebase summary, the text '=== Codebase Summary
===' will appear.

The codebase summary will contain the following files:

+ the full file for the top level .gitignore, build.sh, Cargo.toml, LLMInstructions.md, and UserSpecification.md
+ all of the filenames of all of the top level files, including names of all the top level files in src/
+ for each module, the following will be provided:
	+ the full InternalDependencies.md file
	+ the full PublicAPI.md file
	+ a list of the names of all files in the module, including documentation files

Note that only files which are not listed in the .gitignore should be provided.
Any file that appears in the .gitignore will not be mentioned in the codebase
summary. If the top level files such as Cargo.toml appear in the .gitignore,
that is an error. Additionally, nothing from the app-data, agent-config, or
agent-state folders should be provided.

Note that every sub-folder of the src/ folder needs to be included in the
codebase summary, even if it does not have the required documentation files and
therefore isn't a full module.

Modules will be declared with the following syntax:

```
=== [module path] ===
```

The module path for the top level files will just be "Project Root".

Full files will be provided with the following syntax:

```
--- [filepath] ---
[file data]
```

And lists of filenames will be provided with the following syntax:

```
--- FILENAMES ---
src/example1.rs
src/module/PublicAPI.md
--- END FILENAMES ---
```

Note that the full filepath from the top level of the project is provided for
each file and filename.

### Parsing the LLM Response

The LLM will provide a response that contains a list of files, structured like
this:

```
%%%files
LLMInstructions.md
UserSpecification.md
src/main.rs
src/example_module/PublicAPI.md
src/other_module/mod.rs
src/other_module/InternalDependencies.md
src/other_module/PublicAPI.md
src/other_module/UserSpecification.md
%%%end
```

The contents of each file in that list need to be loaded from the filesystem,
so that the full files can be provided as context. The files will be presented
with the following syntax:

```
--- [filepath] ---
[file data]
```

This list of files becomes the codebase in the next step. This codebase is
logged as 'codebase.txt' in the logs.

Note that it must be strictly enforced the preprocessing LLM cannot request any
file that appears in the .gitignore. The LLM is also not allowed to request any
files in the app-data, agent-config, and agent-state folders. The parser
closely checks that those files have not been requested and will not be
included in the codebase.

When logging, the context_builder will use the identifer that is provided by
the caller.

The caller will tell the context builder what llm model to use.
