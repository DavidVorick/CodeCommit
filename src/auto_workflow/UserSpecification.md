# Auto Workflow

This is a specification for the auto workflow module, a module that looks at
the project state and automatically runs agents to review and update the
codebase. These decisions are made based on progress milestones that are
tracked as the codebase is built out.

The process for determining what to work on is fully scripted, and does not
depend on LLMs. The work itself typically does involve calling LLMs.

## Implementation Progression

The automated workflow splits work into four phases, as well as a
pre-processing phase. Modules are processed one-by-one for each phase, and when
every module has completed each phase, the next phase can begin.

Modules in a code-commit project always form a dependency DAG. One module is
not allowed to import another if it would form a dependency loop. This allows
the auto workflow to process modules in reverse dependency order: first it
processes modules that have no dependencies on other modules (L0 modules), then
it processes modules that exclusively depend on modules with no other
dependencies (L1 modules), and so on.

Modules track their dependencies in a file called ModuleDependencies.md, and it
has this format:

```
# Module Dependencies

src/llm
src/logger
```

The first line is always `# Module Dependencies`, the second line is always
blank, and the remaining lines always list out the full set of other modules
that this module has as dependencies, one per line. This format makes the file
machine-readable.

## Preprocessing Phase

Because the phases make progress based on the module dependency graph, a module
dependency graph has to be built. The graph is always built using the
ModuleDependencies.md file. The graph is built on the fly by loading all
modules and checking their dependency files.

A module can be identified by the presence of a UserSpecification.md file in a
folder. The 'root' module has a UserSpecification.md file at the top level of
the project, and then every other module will appear as a directory or
subdirectory in the src/ folder. Module folders can have infinite depth, so the
full folder structure of the src/ folder needs to be scanned.

The full list of modules is assembled, and then the dependency graph is built.
If a module lacks a ModuleDependencies.md file, then an error is returned
explaining which module is missing a ModuleDependencies.md file.

Once the dependency tree is available, the modules are processed.

## Processing Order

Modules are processed in phases, and each phase has a series of steps. When
processing modules, a module that has not completed an earlier phase always
takes priority over one that has completed the phase. If two modules are on the
same phase, the module with the lowest dependency depth is processed first. If
two modules are on the same phase and have the same depth, they are processed
in alphabetical order. When a module is being processed for a certain phase,
the steps of that phase are processed on that module in order.

Steps are processed using workflows. Each workflow can provide one of three
responses, which will be wrapped in '@@@@' tags on either end for easy machine
parsing. The three potential responses are 'task-success', 'changes-requested',
and 'changes-attempted'.

That means the output string will either contain '@@@@task-success@@@@' or
'@@@@changes-requested@@@@' or '@@@@changes-attempted@@@@'. If none are
present, or if multiple are present, an error is returned to the user.

The workflow may also have a comment section, which will be wrapped by
'%%%%comment%%%%' and '%%%%end%%%%' tags. For example:

%%%%comment%%%%
The UserSpecification is not self-consistent, one section says that there
should be no network calls, and another section says that there should be an
LLM call
%%%%end%%%%

If there are multiple comment sections, an error is returned. The comments are
presented to the user directly in stdout.

If a task is passed, the auto workflow will automatically reset and keep going,
continuing until a task is not passed.

### The Phases and Steps

Phase one has four steps:

1. "self-consistent": ensure the specification is consistent with itself and
   free of any confusing statements.
2. "implemented": ensure there is an implementation of the module that is
   faithful to the specification.
3. "documented": ensure that the ModuleDependencies.md and APISignatures.md
   file is accurate to the actual implementation of the module
4. "happy-path-tested" ensure that happy path testing exists for all code

Phase two has four steps:

1. "dependency-verified": ensure that the implementation is making correct use
   of all dependencies.
2. "secure": ensure that the implementation follows best practices for the
   security model of the module and of the project as a whole.
3. "complete": ensure there are no major gaps in the module's design or
   implementation.
4. "edge-tested": ensure that there is robust testing of all edge cases,
   including adversarial inputs.

Phase three has three steps:

1. "simple": ensure that the code has been simplified as much as possible.
2. "logged": ensure that there is sufficient logging in the module to support a
   production deployment. Depending on the module, logging may not be needed.
3. "integration-tested": ensure that the code has robust integration testing
   that verifies all dependencies.

Phase four has three steps:

1. "benchmarked": ensure that the code has benchmarks which verify that
   performance meets requirements, and that the test suite fails if a benchmark
   is too slow.
2. "fuzzed": ensure that fuzz tests have been written for all functions that
   may require fuzzing. Depending on the module, fuzz testing may not be
   needed.
3. "polished": ensure that all coding best practices are followed throughout
   the implementation.

## Prompt Construction

When constructing the prompt, each section is labeled with the [label] format
prior to the relevant information being provided. The prompt templaates have
already been created by the supervisor and exist within this module.

### Phase 1

1. self-consistent

[response format instructions]
[self consistent prompt]
[top level UserSpecification.md]
[target user specification]

If the target user specification is the top level user specification, then
target user specification is skipped, as it was already provided. For top level
code, the ModuleDependencies.md and APISignatures.md file appear in the src/
directory.

2. implemented - no cached UserSpecification

[response format instructions]
[implementation-no-cache prompt]
[target user specification]
[codebase, including dependency files and top level UserSpecification]

The codebase should include the top level UserSpecification.md, every single
file in the target module, and every single UserSpecification.md and
APISignatures.md file for every dependency. This list can be automatically
assembled, without needing help from an LLM.

The implementation is completed by calling out to the committing-code module.

2. implemented - cached UserSpecification

[response format instructions]
[implementation-with-cache prompt]
[cached target user specification]
[target user specification]
[codebase, including dependency files and top level UserSpecification]

The codebase should include the top level UserSpecification.md, every single
file in the target module, and every single UserSpecification.md and
APISignatures.md file for every dependency. This list can be automatically
assembled, without needing help from an LLM.

The implementation is completed by calling out to the committing-code module.

3. documented

[response format instructions]
[documented prompt]
[target user specification]
[codebase]

The codebase should include every source code file contained within just the
module.

The implementation is completed by calling out to the committing-code module.

4. happy-path-tested - no cached UserSpecification

[response format instructions]
[happy-path-tested prompt]
[target user specification]
[codebase, including dependency files and top level UserSpecification]

The codebase should include the top level UserSpecification.md, every single
file in the target module, and every single UserSpecification.md and
APISignatures.md file for every dependency. This list can be automatically
assembled, without needing help from an LLM.

The implementation is completed by calling out to the committing-code module.

4. happy-path-tested - cached UserSpecification

[response format instructions]
[happy-path-tested prompt]
[cached target user specification]
[target user specification]
[codebase, including dependency files and top level UserSpecification]

The codebase should include the top level UserSpecification.md, every single
file in the target module, and every single UserSpecification.md and
APISignatures.md file for every dependency. This list can be automatically
assembled, without needing help from an LLM.

The implementation is completed by calling out to the committing-code module.

(future phases will be introduced at another time, for now, just phase 1 is
fully specified)

## Specification Caching

Whenever an implementation stage is completed for a UserSpecification, the full
UserSpecification is cached in a file that corresponds to the stage name. This
means that there can be up to 14 different versions of the UserSpecification
cached for each UserSpecification in a CodeCommit project.

The cached UserSpecifications are saved in a folder called agent-state/ at a
corresponding module level. For example, if a project has a UserSpecification
file at UserSpecification.md and src/llm/UserSpecification.md and
src/logger/UserSpecification.md, then the agent-state/ folder will have the the
following file structure:

agent-state/specifications/self-consistent
agent-state/specifications/implemented
agent-state/specifications/...etc
agent-state/specifications/src/llm/self-consistent
agent-state/specifications/src/llm/implemented
agent-state/specifications/src/llm/...etc
agent-state/specifications/src/logger/self-consistent
agent-state/specifications/src/logger/implemented
agent-state/specifications/src/logger/...etc

If no file exists for a given step for a given UserSpecification, that means
that UserSpecification has never reached that implementation stage before.

This cache is used by the auto-workflow module to figure out what steps need to
be taken to advance the project. If the current UserSpecification does not
exactly match the cached UserSpecification for a given stage, then that stage
needs to be revisited by the auto workflow tool.

## Logging

When logging, the name that is passed into the logger is
'auto-workflow-[spec-path]-[stage]', where any pathing characters are replaced
by '+' characters.

## Security

All paths listed in the dependencies must not have any path traversal
characters. If there are path traversal characters such as '..', an error is
returned.
