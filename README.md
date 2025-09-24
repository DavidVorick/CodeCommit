# code-commit

code-commit is an agentic workflow tool that helps developers interface more
seemlessly with LLMs. It is currently targeted at rust codebases, but could
easily be adapted for other codebases.

code-commit is very specification focused. It depends on two files -
UserSpecification.md and LLMInstructions.md to define a goal, and then it uses
LLMs to implement the specifications.

code-commit is constantly evolving and might often be in a state of disrepair.
The best way to make use of code-commit for yourself is to look at the git
tags. Each tagged version is a stable and fully put together iteration of
code-commit (or at least, I thought it was stable when I made the tag).

The project is evolving too quickly for the README to be he all that helpful,
so if you want to use code-commit for yourself I suggest looking at the
UserSpecification.md file. It contains the full specification for code-commit,
and is also a good example of how to use code-commit (as code-commit is itself
a project made with code-commit).
