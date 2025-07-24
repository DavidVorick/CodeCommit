# User Specification

This is a specification for the implementation loop of an agentic coding
workflow. The implementation loop is managed by a binary called 'code-commit'
which will programatically follow the following loop:

1. (Setup) Build context for an LLM that requests an implementation.
2. (Setup) Call the LLM and get a response
3. (Setup) Parse the response and update the corresponding files on disk.
4. (Loop Begins)
5. Run the build for the project.
6. If the build passes, exit with an error code that indicates success
7. Provide all build output to the LLM and ask the LLM to fix the code.

The loop will run a maximum of 8 times. If the build is still not passing after
8 rounds, the attempt is considered to have failed, and the binary will exit
with an error code that indicates a failure.

## Implementation Details

For step one of the binary, the prompt has the following format:

[basic query]
[codebase]

The basic query can be found in query.txt, and the codebase can be found in
codeRollup.txt

For step two, the Gemini 2.5 Pro LLM should be used. The API key can be found
in gemini-key.txt. Gemini will need a system prompt, which can be found in
gemini-system-prompt.txt. The first LLM prompt will therefore have the
following format:

[gemini system prompt]
[basic query]
[codebase]

For step three, the LLM is being provided with these instructions from the
codebase:

```
Your outputs will be used by an automated pipeline. This means that the outputs
must follow strict conventions otherwise they will not be parsed correctly and
the build will fail.

Any thoughts that you wish to present to the user should be presented using
'&&&' syntax. For example:

&&&start
This is an example of text that will be displayed to the user.
&&&end

Any thoughts that you think should be included in future calls to debug the new
code that you wrote should use '%%%' syntax. For example:

%%%start
This is an example of text that will be sent alongside the compiler output if
the new code has failed to build. All other context will be stripped away.
%%%end

Any code modifications must be made by providing the full code file using '^^^'
syntax. Instead of 'start', the filename of the updated code will be used. For
example, to update src/main.rs:

^^^src/main.rs
fn main() {
    println!("example program");
}
^^^end

Finally, if you wish to indicate to the binary that all is well and no code
modifications are needed at all, you can use the '$$$' syntax:

$$$start
This is the message that explains why no action is needed.
$$$end

Because the output is being parsed by scripts, there is no flexibility to stray
from these conventions.
```

The code-commit binary will therefore need to be able to parse all four types of
syntax from the LLM. There may be multiple appearances of each type of syntax,
but they are not allowed to overlap. To best handle this, the binary should
parse the syntax in four passes, one pass for each type of syntax. Each pass
will ignore the other types of syntax.

Any thoughts provided by the LLM using the &&& syntax need to be printed to
stdout immediately. They should also be appended to llm-user-output.txt,
without deleting any previous thoughts that were put there.

Any debugging thoughts provided with the %%% syntax should be held in memory.

Any new code provided by the ^^^ syntax should be placed on disk in the correct
location. The binary needs to verify that the code is not using any path
traversal to ensure that a rogue LLM cannot destory files outside of the git
repo. The binary also needs to ensure that the .git folder is off limits, as
well as the build.sh file. Any attempt by the LLM to do path traversal or
access off-limits files should result in a parsing error.

If there are no code files presented, and also there is no $$$ syntax, an error
should be thrown. Similarly, if there is a $$$ syntax and also code files are
presented, and error should be thrown.

Other major output mistakes should also produce an error. for example, if a
syntax is not terminated correctly, or if multiple implementations for the same
codefile appear, if the different syntaxes overlap or nest in any way, etc.

The parsing needs to be able to handle unusual whitespace placement in the LLM
response, as LLMs are not always consistent about how and where they place
whitespace.

If there is an error during parsing, it is treated as a build error and is
presented to the LLM in the next step the same way that a build error would be
presented.

If no errors are thrown during parsing, the binary proceeds to the next step
and attempts to build the project. The build script can be found at 'build.sh'.
If the build passes, the binary exits with a successful error code. Otherwise,
the binary proceeds to the next step, which is to make another query to the
LLM.

While inside of the loop, new prompts to the LLM have the following format:

[gemini system prompt]
[basic query]
[codebase]
"The above query was provided, and you provided the following data in your response"
[all debugging thoughts from setup]
[all suggested file replacements from setup]
"When the file repalcements were made, the build provided the following output:"
[build output from setup]
"You then provided the following data in your subsequent response"
[all debugging thoughts from loop iteration 1]
[all suggested file replacements from iteration 1]
[build output from iteration 1]
"When the file repalcements were made, the build provided the following output:"
[all debugging thoughts from loop iteration 2]
[... and so on ...]

At some point, either the build will pass or 8 iterations will be completed
without success and the build will fail.

## Logging

All LLM prompts and responses should be saved to disk. Each run of
'code-commit' should create a new folder in the 'logs' directory, with a folder
name that is equal to the timestamp at the start of the binary.

Each LLM query should be put inside of that folder in numerical order, such as
'query-1.txt' and 'query-2.txt', etc. Each response should appear next to the
query alphabetically: 'query-1-response.json', 'query-2-response.json', etc.
Just the text portion of the output should be provided redundantly in its own
file, 'query-1-response.txt', etc. The build output should be placed in a
similar location: 'query-1-build.txt', 'query-2-build.txt', etc.
