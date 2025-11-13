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

## The Evolution of Code-Commit

The original idea for code-commit was to create a simple tool that used LLM
APIs to get some code changes for a repo, and then automatically write those
changes to the repo - alleviating the need for repeated copy-paste using a web
ui.

It then quickly became apparent that it was tedious to run the build script,
figure out what the errors were, and then feed the errors back into the query.
So code-commit became a tool that would iteratively feed build errors back to
the LLM so that it could get things compiling and get the tests passing,
automating a boring part of the coding process.

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
projects to use for tirival matters, code-commit projects became products that
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

This is where we are now, and it seems to be working alright. Things are
generally okay for introducing new features, but it can often be deficient for
refactoring, especially if functions are being moved between files or if
function signatures are changing. The challenge is ensuring that the context
building LLM can see from the mechanical build summary what functions are
called where, and allowing the LLM to see that without using up too much
context.

### What's Next?

So far, it has worked really well to develop code-commit by focusing on the
thing that is most immediately slowing down development. And while there are
three general categories of things that seem to be causing issues, one clearly
rises to the top.

The least important issue is using the LLM to design modules and refactor code.
It's been my experience that between the limitations of the context-builder and
the LLM's natrual tendency to split things up in non-helpful ways, that the LLM
pretty much needs heavily supervised handholding during any code refactor.
Though this could turn into a pretty big long term bottleneck, the truth is
that it doesn't take that much time to manage all of the module design by hand.
So while it's probably the most obviously deficient thing about code-commit
right now, it's also not that time consuming to work around.

The middleground issue is the context building, which works pretty well about
70% of the time, and then otherwise falls on its face and needs clear
instructions about which files need to be included. But, similar to the
refactoring and module design, the LLM seems to be quite responsive to direct
instructions about how to build the context, which means it's pretty easy to
work around this deficiency when it pops up.

The most pressing issue is in the automated prompt / system prompt /
LLMInstructions. The default behaviors for a lot of coding things like error
handling, loggging, and building test suites simply don't scale well within an
agentic codebase, and actually usually don't scale well within codebases in
general.

This can pretty easily be fixed with some custom instructions, but between all
of the different types of coding patterns that the LLM needs to know, you end
up with a really fat stack of custom instructions, which spreads the attention
of the LLM really thin, especially on larger codebases. The context builder
helped a lot, but as the number of instructions increase, attention gets spread
thin and the quality of the code that gets produced goes down. Even worse, some
instructions are non-deterministically overlooked, and that problem grows as
the prompt size grows.

The top proposed solution is to break coding into smaller steps, where each
step refactors the code to adhere to better coding paradigms. The prompts will
need to be constructed carefully enough that there isn't any regression each
time a new layer of improvement is added, but by having one set of prompts for
authoring core logic, another for authoring logging, another for authoring
testing, and so on, allowing the set of instructions for each stage to be
minimal and focused, while still covering the full set of quality requirements
on each code change.

This solution would likely come with the consistency check being woven into the
full coding pipeline, once again pulling the code-commit binary down to a
single command / workflow that produces a bunch of helpful output as it runs.
Before the first step, a consistency prompt would run that focuses entirely on
being certain that the UserSpecification documents made sense and were self
consistent. And then after each coding step, a consistency check would run to
check the quality of the code aspect that is being covered, its consistency
with the UserSpecification, and its consistency with the system prompts for
that coding step.

The consistency check in particular would be given the authority to abort the
coding process, if it felt that something was seriously misaligned or needed
clarification from the user. Aside from that, it would be instructed to provide
output to the user based on what it saw with either general comments or
suggestions, or flagging places where the implementation and the spec diverged.
Or even flagging where the spec diverged from the prompts.

This upgrade would collapse everything back down to a single command, and
perhaps the user query could even provide instructions on how many stages to
run and which stages to run. This would both increase the utility of
code-commit for large scale codebases, and also simplify the UX, improving the
chances that other people find the utility useful.
