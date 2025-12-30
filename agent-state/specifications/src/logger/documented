## Logger

This is the specification for the logger module.

Each call to a workflow creates its own folder in the agent-config/logs/
directory that logs all of the activity performed by that workflow. The folder
for the workflow will be "[date]-[workflow]" using the yyyy-mm-dd-hh-mm-ss date
format. For example, a logging folder might be named
"2025-09-23-19-51-35-committing-code". The API of the logger structurally
enforces that this format is followed.

The logger module takes care of creating the logging folder, and then other
parts of the codebase declare which files they would like to log to within the
folder, and what data they would like to log.
