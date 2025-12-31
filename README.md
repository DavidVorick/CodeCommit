# code-commit

code-commit is a pure-cli agentic coding tool that can make use of any
instruction based LLM as a backend. It is currently targeted at rust codebases,
but could easily be adapted for other codebases.

code-commit is highly opinionated, and is only meant to be used with codebases
that are explicitly code-commit codebases. code-commit believes that LLMs have
certain strengths and weaknesses, and a codebase that is written by LLMs should
be deliberately architected to cater towards their fundamental capabilities.

There is therefore a moderate learning curve to using code-commit, and its most
likely easier to entirely rewrite an existing codebase than it is to port
existing code into a code-commit format.

## The Basics

A code-commit codebase is fully defined by a series of UserSpecification.md
files. The specification is written in plain English, and it describes what the
code is supposed to do. LLMs read these UserSpecification files and then
implement the things that are described.

Critically, before implementing anything, the LLMs run a sanity check on the
specification. If anything is confusing or inconsistent, the LLM will halt,
explain the problem with the spec, and then refuse to write code until the spec
has been cleaned up.

The spec analysis works both on a per-spec basis, and also checks that each
spec makes sense in the context of the rest of the codebase.

Most code-commit projects are split between a minimal set of top-level code and
a bunch of modules. Top level code can import modules, and modules can import
other modules, but the dependency graph must form a DAG. This means that
modules are not allowed to import code from the top level of the repository -
if a module needs code, it has to get that code from another module.

The top level code is intended to have the high level overview of the codebase
and its purpose, and each module is intended to be specific and focused. The
modules create specific APIs that can be used by either other modules or by the
top level code.

Each module contains a ModuleDependencies.md file, which tracks what other
modules get imported and help the automated workflow process the modules in the
right order. And then the other documentation file is APISignatures.md, which
gives quick definitions of each module's exports so LLMs know how to use their
dependencies.

Beyond that, the best way to know how to use the tool is to go read all of the
UserSpecification documents in the code-commit codebase. If you, like me, are
using code-commit to write professional code, you should have a good idea of
what happens, in what order, and why. And the best way to learn all that is to
just read the specifications. It'll also give you a good idea of what a
best-practice code-commit codebase looks like.

## General Philosophy

The general philosophy is that LLMs have a relatively limited context window,
and LLMs also have a relatively limited ability to follow a large amount of
instructions. code-commit therefore tries to enable large scale codebases that
give a relatively small number of instructions and a relatively small amount of
context to each LLM call.

As LLMs get better, 'relatively' changes, but as of late 2025 it's my
experience that if you want really high quality code from a frontier model, you
need to keep your context under about 40,000 tokens and you need to keep your
total instruction count (which is difficult to track) under 200.

This means that high quality codebases need to scale in pieces, and the
maximium size of a piece depends on the quality of the LLM. If a codebase has
lots of specific intructions or best practices, those practices need to be
enforced across multiple calls to LLMs rather than conducted all at once.

Finally, code-commit tries as much as possible to use LLMs exclusively for the
coding itself, relying on normal, non-intelligent scripts and processes for the
actual steps of updating files and running the build. Though it's not
bulletproof, it significantly reduces the chances that an LLM does something
destructive or irreversible to the host machine, because the actions that the
LLM itself can take are highly constrained. For example, only the user can
commit code. An LLM can make changes, but the changes don't enter the git
history until a user has had a chance to sign off.

## Coding Phases

When code-commit works on a codebase, it works in phases. Each phase takes the
project to a deeper layer of maturity, and increases the cost of making
breaking changes.

Within each phase, code-commit works backwards through the module dependency
tree, starting with the modules that have no dependencies (L0 modules), and
then progressing to the modules that only depend on modules with no
dependencies or fewer (called L1 modules), and so on, until every module has
been completed for that phase. code-commit will complete the full phase for
each module before moving onto the next module.

Though the phases have been adjusted over time, the initial progression looked
something like this:

Phase 1:

1. Ensure the specification is sensible
2. Ensure there is a basic implementation that is faithful to the specification
3. Ensure that the public APIs of the module have been properly documented for
   other modules, as well as the dependencies of the module.
4. Ensure that basic happy-path testing exists for all major functions of the
   module.

Phase 2:

1. Ensure the module follows best practices for the project's security model.
2. Ensure there is robust testing of edge cases and adversarial inputs.
3. Ensure that there are no major gaps in the project design or implementation.

Phase 3:

1. Ensure that the code is as simple as possible.
2. Ensure that the code has sufficient logging to debug in production.
3. Ensure that the code has integration testing which verifies its dependencies
   work as needed.

Phase 4:

1. Ensure that the code has benchmarks which verify that performance meets
   requirements.
2. Ensure that fuzz tests exist for any functions that may require fuzz
   testing.
3. Ensure that all coding best practices are followed throughout the
   implementation.

Projects that are just prototypes should stay in phase one, to get the fastest
possible iteration speed. Projects that are ready to be piloted in a real world
environment should stay in phase two. Projects that are being piloted in
production grade environments should stay in phase three, and projects that are
being fully deployed as production-grade professional software should be phase
four.

## Usage

TBD, but the goal is to have just two commands: 'code-commit' and 'code-commit
--query'. 'code-commit' will review the specifications and advance them
automatically, and 'code-commit --query' will allow the user to ask questions
about the codebase.

Because the commands are basic, the user has to develop the app entirely by
modifying the UserSpecification documents.

## The Evolution of Code-Commit

The original idea for code-commit was to create a simple tool that used LLM
APIs to get some code changes for a repo, and then automatically write those
changes to the repo - alleviating the need for repeated copy-paste using a web
ui.

It then quickly became apparent that it was tedious to run the build script,
figure out what the errors were, and then feed the errors back into the query.
So code-commit became a tool that would iteratively feed build errors back to
the LLM so that it could get things compiling and get the tests passing,
automating away another boring part of the coding process.

At that point, it was so easy to write code with LLMs that writing code
yourself started to feel wasteful. Instead, most time was spent writing
specifications, and the LLM would handle the rest. This created a sort of
blindness that felt uncomfortable - how could you feel good about code if you
had never seen it? How would you know that the LLM had implemented the code
that you intended for it?

That gave birth to the 'consistency' workflow, which basically asks the LLM to
provide reviews of the specification and check that the specification is
consistent with itself. It turned out that LLMs are very good at identifying
things they don't understand themselves when prompted to do so, and this
allowed for a much, much deeper level of alignment between the user's intention
and the LLM's deliverables.

The introduction of the consistency workflow is really what allowed code-commit
to start producing high reliability code. Several rounds of specification
review usually led to a code implementation that completed in one-shot, meaning
that for simpler projects, writing the specification was the bottleneck on
getting a working codebase.

This allowed codebases to start to get more ambitious, and instead of one-off
projects to use for trivial matters, code-commit projects became products that
could evolve over time. At least... up until a point. Unless the user was hyper
diligent about updating the specification and implementation in tandem, some
drift would start to occur over time, which put a ceiling on how much a
code-commit project could evolve before starting to stall out.

This led to an evolution of the consistency workflow. Instead of just checking
consistency within the specification, it started to also check for consistency
between the specification and implementation, highlighting places where
mismatches existed, and allowing the user to maintain projects through greater
amounts of growth and adjustment, even if the user was not themselves being
diligent to keep the spec and the implementation aligned.

This lifted the ceiling once more, and it actually lifted the ceiling high
enough to create codebases that started to exceed the useful context of the
frontier LLMs, which at the time was around 100,000 tokens. The models claimed
to be able to go to a million tokens or even 2 million tokens, but in practice
fidelity really seemed to suffer starting somewhere between 60,000 tokens and
100,000 tokens.

Going further would mean doing some context engineering. And while the industry
prefers to use some sort of RAG solution, code-commit found better reliability
by just sending a summary of the codebase to an LLM and then asking the LLM to
select which files to include in the context based on the query.

This ended up being a pretty significant overhaul, because a "codebase summary"
had to be compiled mechanically, without the use of an LLM. And whatever
summary was pulled together mechanically had to be good enough that the
context-building LLM could figure out what context was actually necessary.

After that, the whole code-commit design was overhauled, because the main
bottleneck became UserSpecification design - it got quite finicky for larger
projects. The development process was broken into phases, and a clear module
dependency DAG was established so that modules could be built out one phase at
a time.

This type of separation also significantly simplified the process of building
context for the LLMs, allowing it to be mostly automated. The phase based
architecture is still being worked out, but early signs point to it being very
effective.
