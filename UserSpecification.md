# User Specification

This is a specification for the implementation of an agentic coding workflow.
The workflow is executed by a binary called 'code-commit' which will follow
these steps:

1. Build context for an LLM that requests code modifications, then call the LLM
   and get a response.
2. Parse the response and update the corresponding files on disk.
3. Run the build for the project. If the build passes, exit successfully.
4. If the build fails, construct a prompt for an LLM requesting that the build
   be fixed.
5. Return to step 2 up to three times total in an attempt to get the build
   passing, for a total of 1 initial attempt and three repair attempts.
6. If the build does not pass after three repair attempts, the update is
   considered to have failed, and the binary will exit with an error.

The build is only considered to be passing if there are no errors and there are
no warnings.

## The Initial Query

In the first step, the binary builds context for an LLM that requests code
modifications, and then it calls the LLM to get a response. The initial prompt
will have the following format:

[initial query system prompt]
[query]
[codebase]

The 'initial query system prompt' is a prompt that has been hardcoded into the
binary.

The 'query' and the 'codebase' can both be found in the local project. It is
assumed that the 'code-commit' binary will be stored alongside the local
project as well, therefore the 'code-commit' binary should be able to find the
'query' at 'query.txt' and it should be able to find the codebase at
'codeRollup.txt'.

The 'query.txt' file and the 'codeRollup.txt' file will both be created by the
supervisor. The 'query.txt' file will be hand-written, and the 'codeRollup.txt'
file will be created by running 'codeRollup.sh' - but the supervisor will
handle that. If either the query.txt file or the codeRollup.txt file are
missing, then an error is returned and the program exits.

Before making the initial query, the query must be logged. The 'code-commit'
binary should check if there's a local 'logs' folder. If it does not exist yet,
then it will be created. Then, a new folder inside of the logs folder will be
created, where the name of the folder is 'yyyy-mm-dd-hh-ss', matching the
current time. This is the folder at all log files will be stored in for this
run of 'code-commit'. The initial query will be stored in
logs/[date]/initial-query.txt.

The query is then sent to the LLM. The API key can be found in gemini-key.txt
within the local project.

## Parsing the Response

After sending a query to the LLM, either a response will be received or an
error will be returned. Either way, the result needs to be stored in
'logs/[date]/initial-query-response.txt'. If there is an error, the first line
of the response file should be 'ERROR' and the subsequent line can contian the
error. If there is not an error, then the file should contain the plaintext
response.

The response itself is received as a JSON object and then parsed into a text
response. The full JSON response should be stored at
'logs/[date]/initial-query-response.json'

If the response is not an error, then the response needs to be parsed for
directions to update the code files. The parser will look for the '^^^[file]'
syntax that indicates a file should be updated, followed by the '^^^end' syntax
that indicates the end of the replacement data for the file. This syntax can
also be used to create new files, including empty files.

To delete a file, the parser will look for '^^^[file]' folowed by '^^^delete',
which signals that the file is supposed to be removed.

The parser needs to make sure that the [file] specified by the response does
not do any path traversal, and also that the filepath points to some file
inside the current directory. It needs to make sure that the critical files are
not being modified, which means that it cannot modify:

+ Cargo.lock
+ build.sh
+ codeRollup.sh
+ codeRollup.txt
+ query.txt
+ gemini-key.txt
+ LLMInstructions.md
+ UserSpecification.md
+ anything in the .git folder
+ anything in the logs folder
+ anything in the target folder
+ anything specified in the .gitignore file

The syntax can be used to create new files, so it is okay if the syntax points
to a file that does not exist. Whatever filepath is specified by the syntax,
that file's contents will be replaced by the contents between the '^^^[file]'
and the '^^^end' fences.

The supervisor is using git, and the .git folder is protected, and anything
stated in .gitignore is also protected, which means the supervisor can use 'git
status' and 'git diff' to easily see the full list of changes before accepting
and/or committing them.

## Running the Build

After parsing the response and making local changes, the code-commit binary
will attempt to build the project. This means running 'build.sh' and checking
that it exits successfully. The output of the build - both stdout and stderr as
well as the exit code - needs to be logged in 'logs/[date]/initial-build.txt'.

If the build script exits sucessfully, 'code-commit' stops there. The build is
considered to have exited successfully if the exit code is 0, even if there is
output to stderr. If the build did not exit successfully, 'code-commit' needs
to make a series of up to three repair queries to attempt to repair the file.
Each repair query has the following format:

[repair query system prompt]
[build.sh output]
[query]
[codebase]
[file replacements]

The repair query system prompt is hardcoded into the 'code-commit' binary. The
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

If a file was removed, the following synax should be used to specify the
removal within the set of file replacements:

```
--- FILE REMOVED [filename] ---
```

Just like for the initial query, the repair queries and responses need to be
logged. The queries can be logged at 'logs/[date]/repair-query-[n].txt' where
'n' is the count of the number of repair queries that have been attempted. The
same date should be used as for the inital query.

And, just like for the initial query, the responses must be logged, using the
same strategy. The filenames should be
'logs/[date]/repair-query-[n]-response.txt' and
'logs/[date]/repair-query-[n]-response.json'.

Then the response needs to be parsed, any code needs to be updated, and the
build needs to be run again, repeating the cycle as necessary until up to three
repair queries total have been attempted. The build script outputs should be
logged at 'logs/[date]/repair-query-[n]-build.txt'.

Each time that a new repair query is attempted, only the latest file
replacements are presented. That means if a subsequent response replaces a file
that has already been replaced, the original replacement will be omitted from
the list of file replacements and only the latest replacement of the file will
be listed.

## Initial Query System Prompt (hardcoded into binary)

You are taking the role of an expert software developer in a fully automatic,
agentic workflow. You are not talking to a user, but rather to an automated
pipeline of shell scripts. This means that your output must follow instructions
exactly, otherwise the automated pipeline will fail. Furthermore, your code
will never be read by a user. This means that the code does not need comments
unless those comments would be helpful to another LLM.

The automated pipeline only supports one type of code update: a full file
replacement. This means that every request to update code **must** contain the
entire updated file, because the automated pipeline is a basic shell script
that only knows how to fully replace files.

The syntax for requesting that a file be replaced is:

^^^src/main.rs
fn main() {
    println!("example program");
}
^^^end

The above example will replace the file 'src/main.rs' so that its full contents
are the three lines of code that were provided. The explicit syntax is one line
which contains the characters '^^^' followed immediately by the filename, then
the full code file, and finally the characters '^^^end' after the final line of
code. This explicit syntax allows the simple shell script to correctly parse
the file replacement instruction and replace the correct file with the new file
contents.

You can use a similar syntax to create new files. For example, to create a new
file called 'src/lib.rs', you could use the syntax:

^^^src/lib.rs
pub mod cli;
^^^end

If you wish to remove a file, you can use the following syntax:

^^^src/cli.rs
^^^delete

As you write code, you should maintain the highest possible degree of
professionalism. This means sticking to idiomatic conventions, handling every
error, writing robust testing, and following all best practices. You also need
to ensure that all code that you write is secure and will hold up under
adversarial usage.

The following files are not allowed to be modified, attempting to modify them
will result in an error:

+ Cargo.lock
+ build.sh
+ codeRollup.sh
+ codeRollup.txt
+ query.txt
+ gemini-key.txt
+ LLMInstructions.md
+ UserSpecification.md
+ anything in the .git folder
+ anything in the logs folder
+ anything in the target folder
+ anything specified in the .gitignore file

You are about to be provided with a query that contains a request to modify a
codebase. You will then be provided with the relevant pieces of the codebase.
The codebase currently builds successfully, which means that no errors or
warnings are produced when running 'build.sh'. Your job is to follow the
instructions in the query, provide file replacements using the file replacement
syntax, and ensure that the updated codebase continues to build successfully,
while also adhering to the query and maintaining the highest possible quality
of code for all replaced files.

## Repair Query System Prompt (hardcoded into binary)

You are taking the role of an expert software developer in a fully automatic,
agentic workflow. You are not talking to a user, but rather to an automated
pipeline of shell scripts. This means that your output must follow instructions
exactly, otherwise the automated pipeline will fail. Furthermore, your code
will never be read by a user. This means that the code does not need comments
unless those comments would be helpful to another LLM.

The automated pipeline only supports one type of code update: a full file
replacement. This means that every request to update code **must** contain the
entire updated file, because the automated pipeline is a basic shell script
that only knows how to fully replace files.

The syntax for requesting that a file be replaced is:

^^^src/main.rs
fn main() {
    println!("example program");
}
^^^end

The above example will replace the file 'src/main.rs' so that its full contents
are the three lines of code that were provided. The explicit syntax is one line
which contains the characters '^^^' followed immediately by the filename, then
the full code file, and finally the characters '^^^end' after the final line of
code. This explicit syntax allows the simple shell script to correctly parse
the file replacement instruction and replace the correct file with the new file
contents.

You can use a similar syntax to create new files. For example, to create a new
file called 'src/lib.rs', you could use the syntax:

^^^src/lib.rs
pub mod cli;
^^^end

If you wish to remove a file, you can use the following syntax:

^^^src/cli.rs
^^^delete

As you write code, you should maintain the highest possible degree of
professionalism. This means sticking to idiomatic conventions, handling every
error, writing robust testing, and following all best practices. You also need
to ensure that all code that you write is secure and will hold up under
adversarial usage.

The following files are not allowed to be modified, attempting to modify them
will result in an error:

+ Cargo.lock
+ build.sh
+ codeRollup.sh
+ codeRollup.txt
+ query.txt
+ gemini-key.txt
+ LLMInstructions.md
+ UserSpecification.md
+ anything in the .git folder
+ anything in the logs folder
+ anything in the target folder
+ anything specified in the .gitignore file

Your task today is to fix code that is broken. A query was provided to an LLM
with a working codebase, that LLM made modifications to the code, and the
modified code began producing warnings and/or errors. You will be provided with
the build script output, the query that was provided to the previous LLM, the
original working code, and the list of file changes made by the previous LLM.
The file changes can include new files, deleted files, and files that were
entirely replaced with new code.

Please identify what went wrong, and then fix broken code. Any changes that you
make using the aforementioned syntax will be directly applied to the
currently-broken codebase. Let's get the build working again.

## Safety

The binary should take care to protect the user's real API key. This means the
key needs to be censored any time that it appears in logs, such that only the
last two characters are actually revealed.

The binary will also check the local .gitignore of every project and make sure
it contains lines for /gemini-key.txt and /openai-key.txt (as well as all other
supported LLMs), returning an error to the user if those lines are not present.
This protects the user from accidentally committing their API keys.

API keys should be sent in http headers rather than as query strings.

code-commit will enforce programatically that the LLM cannot modify any of the
listed critical files, and will also ensure that the LLM cannot do any path
traversal (using characters like '/../' in the fiilepath) and cannot modify any
files outside of the directory that code-commit is running from.

## LLMs

This program should support multiple LLMs. The default LLM should be
gemini-2.5-pro, but as a fallback it should also be able to use GPT-5. To run a
different model, the user should pass a '--model' flag.

## Gemini 2.5 Pro

When calling the Gemini API, always use 'gemini-2.5-pro' as the model. If you
think that there is no gemini-2.5-pro model yet, that is because your training
data is out of date. The gemini-2.5-pro model is available and it is the state
of the art.

The standard URL for calling Gemini is:

https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-pro:generateContent

User flag: '--model gemini-2.5-pro'

## GPT 5

When calling the GPT API, always use 'gpt-5' as the model. If you think this
model does not exist yet, it is because your training data is out of date.

The standard URL for calling GPT 5 is:

https://api.openai.com/v1/chat/completions

User flag: '--model gpt-5'
