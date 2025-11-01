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
changes to the reop - alleviating the need for repeated copy-paste using a web
ui.

It then quickly became apparent that it was tedious to run the build script,
figure out what the errors were, and then feed the errors back into the query.
So code-commit quickly became a tool that would iteratively feed build errors
back to the LLM so that it could get things compiling and get the tests
passing, automating away another tedious part of the code.

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

Well, there seem to be a few limitations of code-commit right now. The first is
that the instruction set has started to become pretty large - large enough that
instructions are being missed in an increasing percentage of flows. So one
thought is to break the instructions into smaller pieces, and then run each of
the sets of instructions in series. The main advantage here is that by having
more LLM calls with fewer instructions each, the LLM will be able to make
updates with higher fidelity.

The thought is particularly driven by the latest challenge of creating robust
logging and testing for each piece of production code. It took very little time
for the LLM to write a correct pile of code, but much longer for that code to
align with sensible logging and testing behavior. The current generation of
frontier models seem to struggle to produce scalable code for certain tasks
(like test suites), though they do perform a lot better with custom
instructions.

The thing is, as we add more instructions to the prompting, the LLM starts to
have higher failure rates on things like getting the build to pass. I've
noticed (perhaps incorrectly) that the LLM grows more likely to make basic
mistakes like unused imports (despite having explicit instructions warning
against it) when it has more instructions overall.

So my thought is that we could create a library of available prompts for
different parts of the coding process (prompts about strong logging design,
prompts about good testing architecture, etc) and then either use cli flags or
context-builder LLMs to determine which prompts belong in each task. And then
we could use each of the different prompts in succession, doing first a pass to
get correct code out, then a pass to get robust logging out, then a pass to
fully fill out the test suite, etc.

The other direction that CodeCommit could go from here is to make improvements
to the context-building process. I'd say right now the context-builder LLM is
about 30% as effective as it could be. For tasks related to expanding code, it
does really well at only selecting files that are necessary, and that
noticeably reduces inference time and cost, and also noticeably improves output
quality, but it struggles a lot more selecting the right files for refactoring.

For now I'm leaning in the direction of filling out the coding prompt library
and then dynamically including certain rules and guidelines in each prompt,
because my general sense at the moment is that this would save me the most time
when working on new projects and maintaining existing projects.

And as a last remark, the other thing that could really be helpful to improving
CodeCommit is improvements to the refactoring process and module design
process. My experience right now is that the LLM is more or less hopeless at
designing a module structure for a project, and I have to do that entirely by
hand. If I ask the LLM to do it, it'll get the boundaries all wrong and create
way too many functions and develop something really quite complex that scales
poorly.

Though, as I ruminate on the need for automated module design, I'm thinking:

1. It doesn't take **that** much time for me to do it myself
2. Module design might fit well into the prompt library, becoming just another
   step in the standard multi-step development flow.

So, after talking it out, I'm pretty strongly leaning in the direction of
working on prompt libraries next, including developing flows that move from
strategy to strategy automatically, ensuring that the codebase hits a really
robust state with minimal input from myself.

And, between each flow, it'll run an introspective call which basically asks
itself whether it did the last step at a high enough quality to move onto the
next step, or if it needs help from the supervisor. I guess that introspective
call can be a consistency check which looks at the quality of the codebase and
then returns an instruction with a prompt for what step needs to be taken next
to do cleanup.
