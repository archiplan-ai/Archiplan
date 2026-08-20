---
name: archi-migrate-world
description: Give a standing archiplan project its world facts — read the world prose its intents already hold, the story blocks its old plans authored and the suites it already runs, interview one candidate at a time by offering concrete options, and write a fact only where a person can name the workaround. Use when a project modeled before the world must record the outside conditions its spec assumes.
---

> **Skill freshness — the first step.** In an initialized project, run
> `archi sync-skills` before anything else. The report names
> `.claude/skills/archi-migrate-world/SKILL.md`. When the act is
> `updated` or `created`, the text you follow is stale. Read that file
> again, follow it, and only then continue. `ok` means continue.

> Retrieval — how to find anything here — is the `archi-search` skill.

# Migrate a standing project into the world

A project modeled before the world already claims things about the world.
The claims sit in the opening prose of its intents, in the free-text
story blocks its old plans authored, and under the tests its suites
already name. No verb reads them there, and nothing in the repository
can make them false. This skill moves the ones a person can stand behind
into `archi/world/`, one at a time, by interview.

It deletes nothing. The intent keeps its paragraph, the old plan keeps
its stories, and the world stands beside them. The run returns two
things: the facts that landed, and a brief of what did not map and why.

Judgement is the whole job here, and judgement is why this is a skill
and not a converter. Naming the condition under a story is a person's
call. A converter would have written fluent facts nobody observed, which
is worse than an empty world, because an invented fact reads exactly like
an observed one.

## 1. Read what the project already claims

Run these in the project. They give the candidate list and what already
stands:

```sh
archi world ls                       # the world today; often empty
archi search <phrase> --kind intent  # the world prose, by phrase
ls archi/requirements/*/             # the intents, one folder each
ls archi/plans/*/scenarios.md        # the story blocks, if the tree has old plans
ls tests/ */tests/ test/             # the suites, wherever this project keeps them
```

Three places hold candidates, and nothing else does:

- **The intents.** The opening paragraph of
  `archi/requirements/<intent>/<intent>.md`. A sentence about people,
  their days, their machines or their money is a candidate. A sentence
  about what the system must do is not: that is a requirement and it
  already has a home.
- **The old plans.** `archi/plans/<name>/scenarios.md`, written before a
  plan was forbidden to author stories. No verb reads those blocks. Each
  story that rests on an unstated condition is a candidate.
- **The test suites.** A suite named for a behavior is a behavior
  somebody thought worth pinning, and the condition under it is a
  candidate. Read the names first — the file names, the module names,
  the test names: `offline_open`, `a_retry_after_the_timeout`,
  `a_half_written_upload`. Then read what the test sets up before it
  acts: a clock that jumps, a link that drops, a payload that arrives
  truncated, a limit somebody chose a number for. The test says what the
  system does; the candidate is what the world does to it. A test that
  pins an internal contract — a parser, a formatter, an error string —
  carries no condition and is not a candidate.

Write the candidate list down, each with the file it came from, and work
down it. One candidate is one interview. Never batch them: a batch is a
converter with extra steps.

## 2. The interview — the workaround is the gate

A person answers. You ask, you record, and you never fill an answer in
yourself. For one candidate, in this order:

1. **The condition.** What is true outside the system? Say it in the
   words of the world, not of the model.
2. **The workaround.** Ask what people do today instead of this. Wait
   for a concrete answer: what they do, how long it takes, what it
   costs them.
3. **The behavior.** Which scenarios does the condition dictate? One is
   enough to start.
4. **The reach.** Which parts of the model does the story touch?

**The gate is question 2.** A condition nobody can name a workaround
for is a wish, and only a wish: it belongs in no file. When the operator
cannot name one, this skill writes nothing for that candidate. It goes
in the brief with the sentence it came from, and you move to the next
one. Do not soften the question, do not answer it from the prose, and
do not mint a skeleton "to fill in later". An empty world is an honest
state. A fact nobody observed is not.

The same answer is what can end the fact later, so nobody is asked to
predict the end. A person who writes a fact has already decided to
build, and a guess about the future made there is made at the worst
moment for guessing. The workaround is an observable instead: watch
whether people still do it, and the day they stop, the fact is dead.

**Ask by options, never by a bare open question.** Every question above
has shapes, so put the shapes on the table. Ask through the poll tool
(AskUserQuestion) with two or three concrete candidate answers — the
condition as the prose states it, the same condition one degree
stronger, the same one degree weaker — and let the operator choose. An
open question in the abstract stalls: "what do people do instead?",
asked cold, reads as a riddle, and what comes back is "I do not
understand the question". The same question with three answers beside it
— a manual step, a second tool, a habit that costs an hour a week — is
answered in seconds. Say plainly what the options are: scaffolding for
the operator's own thinking, not a menu. The best answer of a run is
often the fourth one, the one the operator writes after seeing that none
of the three fits.

**Ask again after an apparent axiom.** The first "people just live with
it" is not a verdict, and neither is "nothing would make this false": a
claim that looks like an axiom is far more often a claim stated badly.
Ask for the workaround a second time with shapes — name two things
people could be doing instead and one that would only dent the trouble,
and ask which of them the operator sees. Two outcomes follow and both
are right. The second ask names a real workaround, the candidate becomes
a fact, and the world gains a condition somebody can watch. Or it
confirms a premise that truly cannot fail here — and a premise is not a
world fact: it stays in the intent that already holds it, and the brief
says so. Neither outcome is the mistake. The mistake is stopping at the
first answer.

## 3. Write the fact

One candidate that passed the gate is one file:

```sh
archi world add "<the fact in one line>"
```

The verb mints `archi/world/facts/<slug>.md` with the three lists empty
and the headings in place. `facts/` is the strict layer of the world, and
it is the only one this skill writes into. You write the prose:

- **The name** is the fact in one line, as the operator said it.
- **The paragraph under it** states what condition this is and why the
  behavior follows from it.
- **`## What people do instead`** is the answer to question 2, written
  down: what people do today because the condition holds, how long it
  takes them, and what it costs them.
- **`## Scenarios`** holds the behavior the condition dictates. A
  `### <name>` heading opens one scenario, and `Given`, `When`, `Then`
  and `And` open its step lines; those four are the whole vocabulary.
  The heading text is the scenario's name and its address, so a later
  code-link anchors to it, and the code the link points at says where
  the scenario runs. A `Feature:` or a `Scenario:` line is refused,
  because the fact's own title is the feature and the heading is the
  scenario.

One fact, written out — the operator's sentence as the name, and one
scenario under it:

```markdown
# Trains lose the signal

The carriage drops the network for minutes at a time, so a reader on the
move works from what the device already holds.

## What people do instead

Readers load what they mean to read before they board, and the ones who
forget re-read whatever is still open. It costs them the ten minutes
before the train and the article they wanted.

## Scenarios

### The reader opens the app with no network

Given the device has no network
When the reader opens the app
Then the last synced view appears
```

Write the fact **without the nouns of the model**. The condition is
about the world, so the world's words are the right ones, and `check`
reports `world_speaks_the_model(<element>)` when a model name leaks into
the name or the paragraph.

The fact also names no person and quotes nobody. A migration reads
sentences somebody wrote and hears answers somebody gave, and the fact
carries neither: it says how the world is, as a reader who was not in
the room would say it — not that somebody disliked a thing, not what
somebody said, and never in their words. Who saw it is `sources`, and
words the operator hands over become a file under
`archi/world/resources/` that `sources` then names. No check holds this
half.

The header points three ways:

- **`covers`** takes the elements the story touched, by absolute path.
  Leave it empty when the model has not reached the fact. `check`
  reports `world_uncovered` for that, and it is early work, not a
  defect.
- **`sources`** is empty on a migration run unless the operator hands
  material over, because a claim lifted from prose carries no source
  until somebody goes and looks. Each entry is a path from the project
  root to a file under `archi/world/resources/` — and nothing else
  resolves. The intent the sentence came
  from is not a source: the spec is what the world conditions, so a
  fact grounded in a requirement grounds itself in what it explains,
  and `check` refuses the path. A ticket id, a drive link or a
  recording nobody here can open is refused for the other half of the
  same reason: it is a claim about evidence, not evidence. Material the
  operator hands over is carried into `archi/world/resources/` first,
  and the entry then names that file. Otherwise leave the field empty.
  `check` reports `world_ungrounded`, which is the true state of the
  fact, and the file the claim came from is recorded in the brief.
- **`uses`** names another fact this one holds only while that one
  holds. Empty is the normal case.

Then leave the origin alone. Never delete the intent's paragraph, never
delete or trim `scenarios.md`, and never edit either to point at the new
file. The claim now stands in two places, and that is the accepted
price.

## 4. Check, then hand back the brief

Run `archi check` and report what it said, verbatim in substance:

- Errors block. An unresolved `covers` entry, a `sources` entry that
  reaches no file or points outside `archi/world/`, or a scenario
  outside the grammar is a fact that is not finished. Fix it now.
- The closing line counts the world: `world — <n> facts · <m>
  ungrounded`. A migration raises `m` by one for every fact it writes,
  because prose is not an observation. That number is the measure of
  how much of the world still waits for somebody to look, and it is a
  worklist, not a defect to hide.
- Findings are the worklist, never a reason to stop.

Read the world back the way a reader will: `archi world ls`, and `archi
world ls --covers <element>` from a node to the conditions that rule it.

Then write `world-migration-brief.md` at the project root and tell the
operator it is there. It holds:

- **What landed** — one line per fact: the slug, the file it came from,
  and the workaround the operator named. The header holds no origin, so
  this line is the only record of where the claim was read.
- **What did not map, and why** — every candidate that stopped at the
  gate, each with its sentence and the file it sits in. Name the reason
  in the operator's terms, not as a verdict.
- **What is thin** — facts with one scenario, facts with an empty
  `covers`, and workarounds the operator named but could not cost.
- **Where the prose still stands** — the intents, the `scenarios.md`
  files and the suites this run read and left untouched.

A later run starts from that brief. The candidates that stopped at the
gate are the ones to ask about again, when somebody has watched the
work.
