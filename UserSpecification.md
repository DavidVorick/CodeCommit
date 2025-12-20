dependencies:
  src/auto_workflow
  src/committing_code
  src/consistency
  src/logger

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

## Initialization

The code-commit binary supports an 'init' instruction `code-commit init
<project-name>` which will create all of the basic files needed to kick off a
new code-commit project. Simple/basic versions of the following files will be
created:

+ agent-config/
+ agent-config/logs/
+ agent-state/
+ .gitignore (matching the CodeCommit project .gitignore)
+ build.sh (matching the CodeCommit project build.sh)
+ Cargo.toml
+ src/
+ src/main.rs
+ UserSpecification.md (word wrapped to 80 characters)

When the Cargo.toml file is created, the `name = "code-commit"` line will need
to be replaced with the appropriate project name based on the input command.
If the user doesn't provide a project name, an error will be returned without
making any changes.

After the command completes, instructions are given to the user to drop a
gemini-key.txt and an openai-key.txt into the agent-config folder.

This command is non-agentic. If any of the files already exist, they will be
left untouched, and only the missing files will be created.

## Supported Workflows

The code-commit binary supports multiple workflows, each of which can be
triggered with a command line flag. It is an error to trigger more than one
workflow at a time.

### Committing Code

The 'committing-code' workflow uses LLMs to run programming tasks and is the
default workflow of the 'code-commit' binary, meaning it will be called if no
other workflow flags are provided. Other workflows can be specified with flags,
and if no workflow flags are provided the binary will assume that it is
supposed to execute the workflow for committing code.

When this workflow is triggered, the binary will prompt the user for a query
using their default text editor. This query is then passed to the LLM as part
of the agentic workflow.

By default, the committing-code workflow will produce an error if there are
uncommitted changes to the source code of the repo. This is intended to prevent
the user from accidentally overwriting changes they want to keep. The user can
override this protection with the `--force` flag.

The programmatic slug that refers to this workflow is 'committing-code'.

The flag '--commit' can also be used to trigger this workflow.

### Auto Workflow

The 'auto-workflow' workflow uses LLMs to run a series of programming tasks in
an automated pipeline, delivering a comprehensive milestone pipeline that
verifies specifications, delivers implementations, authors tests, and overall
ensures that code is brought to the highest level of quality across a deep
series of automated prompts.

The programmatic slug that refers to this workflow is 'auto-workflow'.

The flag '--aw' can be used to trigger this workflow.

### Checking Consistency

The 'consistency' workflow uses LLMs to verify that the project is self
consistent. It does not write any code, but rather produces output for the user
that tells them about the state of their project.

More specifically, it looks for places where the UserSpecification is
inconsistent with itself, and it looks for places where the UserSpecification
is inconsistent with the current implementation. It then compiles a report. The
report must be human readable in a simple text editor.

The consistency workflow can be triggered with the command line flag
'--consistency-check' or '--consistency' or '--cc'. The programmatic slug that
refers to this workflow is 'consistency'.

When this workflow is triggered, the binary will prompt the user for a query
using their default text editor. This query is then passed to the LLM as part
of the agentic workflow.

The output of the consistency check will be printed to stdout.

### Rolling Up Code

If code-commit is run with the `--rollup` flag, it will trigger a non-agentic
workflow to create a file called codebase.txt, which will contain every file
tracked by git except for the Cargo.toml and any files in the agent-state/
folder. If the flag `--rollup-full` is used instead, then the codebase.txt file
will additionally contain the Cargo.toml file.

A log file will also be created in the agent-config/logs/ folder.

The programmatic slug that refers to this workflow is 'rollup'.

## LLMs

CodeCommit supports multiple LLMs. The default LLM is gemini-3-pro-preview, and
other LLMs are also supported. To run a different model, the user should pass a
'--model' flag.

If you believe that some of the models mentioned below might not exist, it is
because your training data is out of date.

### Gemini 3 Pro Preview

User flag: '--model gemini-3-pro-preview'

API Key Location: agent-config/gemini-key.txt

### Gemini 2.5 Pro

User flag: '--model gemini-2.5-pro'

API Key Location: agent-config/gemini-key.txt

### GPT 5.2

User flag: '--model gpt-5.2'; '--model gpt-5' is also an alias for GPT 5.2

API Key Location: agent-config/openai-key.txt

## Logging

When any of the workflows are running, they will be logging their activity in
the agent-config/logs/ directory.

The core logic for interfacing with the logs is in the 'logger' module.

## Safety

To ensure that the private data of code-commit projects is never exfiltrated or
exposed, files in the app-data/ folder, the agent-state/ folder, and
agent-config/ folder are never allowed to be provided as input to an LLM, and
also an LLM is not allowed to directly request modifications to those files.
The non-LLM parts of the automated workflows are however allowed to modify
these files.

The app-data/ folder is optional, and not all CodeCommit projects will have it.
