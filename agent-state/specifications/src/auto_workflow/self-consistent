# Auto Workflow

This is a specification for the auto workflow module, a module that looks at
the project state and automatically determines which workflow needs to be run.
These decisions are made based on progress milestones that are provided by
other workflows, which means the decision for which workflow to run does not
require any LLM oversight.

## Implementation Progression

Each UserSpecification file has an implementation progression which establishes
how much progress has been made in completing the specification. Each stage of
progression is associated with a workflow that can be used to complete that
stage.

The stages are as follows:

1. "self-consistent": Is the UserSpecification consistent with itself and free
   of any confusing statements?
2. "project-consistent": Is the UserSpecification consistent with the other
   UserSpecifications in the project and free of any confusing statements? Does
   it properly call out all dependencies? Are requirements properly chained
   into their dependencies?
3. "complete": Is the UserSpecification fully specified and contains all
   critical details that would be necessary for successful implementation,
   without any room for confusion? Are all required performance targets called
   out? Is the logging strategy sufficient?
4. "secure": Is the UserSpecification properly considering all security issues,
   such that it will be robust in an adversarial environment?
5. "implemented": Has the UserSpecification been fully implemented?
6. "focused": Does the implementation include any features that are not
   declared in the UserSpecification?
7. "minimized": Has all of the code been broken into files below 300 LoC in
   size, and functions below 100 LoC in size?
8. "simplified": Has the dependency graph of functions within the
   implementation been reduced to a minimal state?
9. "standardized": Does the implementation meet all code quality requirements?
10. "benchmarked": Does the implementation have robust benchmarks to measure
    performance?
11. "logged": Is there an appropriate level of logging throughout the
    implementation?
12. "optimized": Have the UserSpecification and implementation been optimized
    to the correct level? Are there any last implementation changes required to
    bring the implementation to the highest level of quality?
13. "happy-path-unit-tested": Are there happy-path unit tests for every
    function in the implementation?
14. "edge-case-unit-tested": Are there thorough and robust unit tests probing
    every branch and edge case for every function in the implementation?
15. "locally-integration-tested": Are there thorough end-to-end tests for all
    of the code in the implementation?
16. "locally-fuzzed": Have fuzz tests been written for any functions or
    features that may benefit from fuzz testing?
17. "dependency-integration-tested": Are there thorough end-to-end tests which
    also verify that all dependencies are working as required?

Note that the first 16 steps are all internal, and therefore never need to be
updated unless the UserSpecification for the local module has been updated,
however the 17th step is external, and therefore needs to be updated any time
that the specification for one of the dependencies changes in a way that is
relevant to the module's implementation.

### Prompt Construction

When constructing the prompt, each section is labeled with the [label] format
prior to the relevant information being provided.

1. self-consistent

[response format instructions]
[self consistent prompt]
[target user specification]

2. project-consistent

[response format instructions]
[project-consistent prompt]
[target user specification]
[parent user specification]
[all child user specifications]

When putting together the prompt for the project-consistent stage, the target
user specification is provided first, followed by the parent user
specification, followed by all child user specifications.

The parent user specification is the specification in the parent folder, and
the child user specifications are all user specifications located in child
folders.

3. complete

[response format instructions]
[complete prompt]
[target user specification]
[parent user specification]
[all child uesr specifications]

When putting together the prompt for the project-consistent stage, the target
user specification is provided first, followed by the parent user
specification, followed by all child user specifications.

The parent user specification is the specification in the parent folder, and
the child user specifications are all user specifications located in child
folders.

4. secure

[response format instructions]
[secure prompt]
[target user specification]
[parent user specification]
[all child uesr specifications]

When putting together the prompt for the project-consistent stage, the target
user specification is provided first, followed by the parent user
specification, followed by all child user specifications.

The parent user specification is the specification in the parent folder, and
the child user specifications are all user specifications located in child
folders.

## Specification Caching

Whenever an implementation stage is completed for a UserSpecification, the full
UserSpecification is cached in a file that corresponds to the stage name. This
means that there can be up to 17 different versions of the UserSpecification
cached for each UserSpecification in a CodeCommit project.

The cached UserSpecifications are saved in a folder called agent-state/ at a
corresponding module level. For example, if a project has a UserSpecification
file at UserSpecification.md and src/llm/UserSpecification.md and
src/logger/UserSpecification.md, then the agent-state/ folder will have the the
following file structure:

agent-state/specifications/self-consistent
agent-state/specifications/project-consistent
agent-state/specifications/...etc
agent-state/specifications/src/llm/self-consistent
agent-state/specifications/src/llm/project-consistent
agent-state/specifications/src/llm/...etc
agent-state/specifications/src/logger/self-consistent
agent-state/specifications/src/logger/project-consistent
agent-state/specifications/src/logger/...etc

If no file exists for a given step for a given UserSpecification, that means
that UserSpecification has never reached that implementation stage before.

This cache is used by the auto-workflow module to figure out what steps need to
be taken to advance the project. If the current UserSpecification does not
exactly match the cached UserSpecification for a given stage, then that stage
needs to be revisited by the auto workflow tool.

## Task Selection

When the 'code-commit' command is run, the auto workflow tool will iterate over
every UserSpecification in the project and see which stages have been completed
for the latest version of each UserSpecification. It will target the
UserSpecification that has the least progress and then trigger the appropriate
LLM workflow to make progress on that UserSpecification. If multiple
UserSpecifications are at the same progression level, they are processed in
alphabetical order.

Each workflow can provide one of three responses, which will be wrapped in
'@@@@' tags on either end for easy machine parsing. The three potential
responses are 'progression-complete', 'changes-requested', and
'changes-attempted'.

That means the output string will either contain '@@@@progression-complete@@@@'
or '@@@@changes-requested@@@@' or '@@@@changes-attempted@@@@'. If none are
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

## Safety

To ensure that LLMs adhere to the structure of code-commit progressions, LLMs
must never be allowed to modify data in the agent-state/ folder, even though
that folder is included in the git repo.
