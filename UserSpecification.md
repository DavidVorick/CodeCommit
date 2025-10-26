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

## Committing Code

This is a specification for the implementation of an agentic coding workflow
that follows these steps:

1. Build context for an LLM that requests code modifications, then call the LLM
   and get a response.
2. Parse the response and update the corresponding files on disk.
3. Run the build for the project. If the build passes, exit successfully.
4. If the build fails, construct a prompt for an LLM requesting that the build
   be fixed.
5. Return to step 2 up to three times in an attempt to get the build passing,
   for a total of 1 initial attempt and three repair attempts.
6. If the build does not pass after three repair attempts, the update is
   considered to have failed, and the binary will exit with an error.

The build is considered to be passing if and only if build.sh exits with an
error code of 0.

### The Initial Query

In the first step, the binary builds context for an LLM that requests code
modifications, and then it calls the LLM to get a response. The initial prompt
will have the following format:

[project structure system prompt]
[code modification instructions system prompt]
[initial query system prompt]
[query]
[codebase]

The system prompts can be found in the system-prompts module.

The 'query' and the 'codebase' can both be found in the local project. It is
assumed that the 'code-commit' binary will be stored alongside the local
project as well, therefore the 'code-commit' binary should be able to find the
'query' at 'agent-config/query.txt' and it should be able to find the codebase
at 'agent-config/codeRollup.txt'.

The query file and the code rollup file will both be created by the supervisor.
The query file will be hand-written, and the code rollup file will be created
by running 'codeRollup.sh' - but the supervisor will handle that. If either the
query file or the code rollup file are missing, then an error is returned and
the program exits.

The query is then sent to the LLM.

### Parsing the Response

After sending a query to the LLM, either a response will be received or an
error will be returned. If the response is not an error, then the response
needs to be parsed for directions to update the code files. The parser will
look for the '^^^[file]' syntax that indicates a file should be updated,
followed by the '^^^end' syntax that indicates the end of the replacement data
for the file. This syntax can also be used to create new files, including empty
files.

To delete a file, the parser will look for '^^^[file]' followed by '^^^delete',
which signals that the file is supposed to be removed.

The parser needs to make sure that the [file] specified by the response does
not do any path traversal, and also that the filepath points to some file
inside the current directory. It needs to make sure that the LLM is not
attempting to modify critical files, namely:

+ .gitignore
+ Cargo.lock
+ build.sh
+ codeRollup.sh
+ LLMInstructions.md
+ any file named UserSpecification.md
+ anything in the .git folder
+ anything in the agent-config folder
+ anything in the target folder
+ anything specified in the .gitignore file

The syntax can be used to create new files, so it is okay if the syntax points
to a file that does not exist. Whatever filepath is specified by the syntax,
that file's contents will be replaced by the contents between the '^^^[file]'
and the '^^^end' fences.

The supervisor is using git, and the .git folder is protected, and anything
stated in .gitignore is also protected, which means the supervisor can use 'git
status' and 'git diff' to easily see the full list of changes by the LLM before
accepting and/or committing them.

The parser should do a verification pass before making any file modifications.
If any part of the LLM response attempts to modify a disallowed file, then no
files should be updated on disk at all.

### Running the Build

After parsing the response and making local changes, the code-commit binary
will attempt to build the project. This means running 'build.sh' and checking
that it exits successfully. The output of the build - both stdout and stderr as
well as the exit code - needs to be logged in the logging folder with the name
"build.txt". A numerical prefix needs to be added to the file name so that it
is properly grouped with the corresponding LLM call.

If the build script exits successfully, 'code-commit' stops there. The build is
considered to have exited successfully if the exit code is 0, even if there is
output to stderr; some build processes provide non-warning informational output
to stderr. If the build did not exit successfully, 'code-commit' needs to make
a series of up to three repair queries to attempt to repair the file.  Each
repair query has the following format:

[project structure system prompt]
[code modification instructions system prompt]
[repair query system prompt]
[build.sh output]
[query]
[codebase]
[file replacements]

The repair query system prompt can be found in the system-prompts module.  The
build.sh output is the entire output (including both stdout and stderr)
provided when running build.sh. The query is the contents in query.txt (which
have not been modified), the codebase is the codebase found in codeRollup.txt
(which has not been modified), and the file replacements are all of the files
that got replaced by the system parser.

The file replacements should be presented with the following syntax:

```
--- FILE REPLACEMENT [filename] ---
[file data]
```

If a file was removed, the following syntax should be used to specify the
removal within the set of file replacements:

```
--- FILE REMOVED [filename] ---
```

Then the response needs to be parsed, any code needs to be updated, and the
build needs to be run again, repeating the cycle as necessary until up to three
repair queries total have been attempted. The build script outputs should be
logged as build.txt, with an appropriate numerical prefix so each build.txt
file is properly grouped with its corresponding LLM call.

Each time that a new repair query is attempted, only the latest file
replacements for each file are presented. That means if a subsequent response
replaces a file that has already been replaced, the original replacement will
be omitted from the list of file replacements and only the latest replacement
of the file will be listed.

### Safety

The binary should take care to protect the user's real API key. This means the
key needs to be censored any time that it appears in logs, such that only the
last two characters are actually revealed.

The binary will also check the local .gitignore of every project and make sure
it contains lines for /agent-config to ensure that API keys are not going to be
accidentally committed a repo. An error will be returned if the /agent-config
line is not present in the .gitignore.

API keys should be sent in http headers rather than as query strings.

code-commit will enforce programatically that an LLM cannot modify any of the
listed critical files, and will also ensure that the LLM cannot do any path
traversal (using characters like '/../' in the filepath) and cannot modify any
files outside of the directory that code-commit is running from.

## Checking Consistency

This is a specification for the implementation of an agentic coding workflow
that builds context for an LLM that reviews code and then generates a report.
The report is placed as a standalone file in
agent-config/consistency-report.txt.

### The Query

The binary builds context for an LLM that requests code review, and then it
calls the LLM to get a response. The prompt will have the following format:

[project structure system prompt]
[consistency check system prompt]
[query]
[codebase]

The query is then sent to the LLM, and the text response is recorded in
agent-config/consistency-report.txt.
