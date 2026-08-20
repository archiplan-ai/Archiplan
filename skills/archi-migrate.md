---
name: archi-migrate
description: Migrate a standing archiplan project past the mechanisms it predates — measure the gap first (`archi world ls` empty against a standing model, `archi link ls` counting rows under `inferred`, `ls -d archi/plans/*/waves` hitting under completed plans), then run the pass the measurement names: the world interview that turns prose a person can stand behind into world facts, the journal triage that retires what the old matcher guessed, or the scrap sweep that removes the `waves/` folders an older binary's closes left behind; a tree holding `.fractal/` goes to `archi-migrate-fractal` instead.
---

> **Skill freshness — the first step.** In an initialized project, run
> `archi sync-skills` before anything else. The report names
> `.claude/skills/archi-migrate/SKILL.md`. When the act is `updated` or
> `created`, the text you follow is stale. Read that file again, follow
> it, and only then continue. `ok` means continue.

> Retrieval — how to find anything here — is the `archi-search` skill.

# Migrate a standing project

A project modeled before a mechanism carries a gap the tool can measure,
and the measurement names the pass to run:

- `archi world ls` — empty while a model stands is a project from before
  the world. Run the world pass.
- `archi link ls | awk '{print $5}' | sort | uniq -c` — rows counted
  under `inferred` are a journal from before the declaration. Run the
  journal pass.
- `ls -d archi/plans/*/waves` — a hit under a plan that
  `archi plan list` shows completed is scrap from a binary older than
  `the-plan-cleans-up-after-itself`. Run the scrap pass.

A tree holding `.fractal/` belongs to the old client, and that migration
— the binary swap and the import — is its own page: the
`archi-migrate-fractal` skill.

## The world pass — migrate a standing project into the world

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

### 1. Read what the project already claims

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
down it. Drafting them all up front is fine — a batch of drafts is
reading, and reading is this step. One candidate is one interview.
Never batch them: a batch of interviews is a converter with extra
steps.

### 2. The interview — the workaround is the gate

A person answers, and the prose speaks first. Draft the candidate whole
from the file it came from — the condition, the behavior, the reach:
the sentence that made it a candidate usually answers all three. Put
the draft in front of the operator to confirm or correct, and ask only
what the prose does not answer. A draft is a proposal, not a record:
the operator's yes or correction is what lands in the file. The one
answer never taken from prose is the workaround — ask it of every
candidate, because the gate is worth nothing answered from paper. For
one candidate, in this order:

1. **The condition.** What is true outside the system? Say it in the
   words of the world, not of the model.
2. **The workaround.** Always asked, never drafted. Ask
   what people do today instead of this. Wait for a concrete answer:
   what they do, how long it takes, what it costs them.
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

### 3. Write the fact

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

When a standing suite already proves a scenario — often the very test
the candidate came from — bind the two at the moment the fact is
written:

```sh
archi link add "<fact>#<scenario>" <test file>#<test fn> --kind indirect
```

The fact arrives holding proof the tree already runs. Do not wait for
a plan close that may never come: when one does, this link is what it
finds already standing.

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

### 4. Check, then hand back the brief

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

## The journal pass — migrate a journal written by the matcher

A project modeled before the declaration carries a journal capture wrote by
itself. That capture minted a link wherever the name of a changed file shared
one word with the name of a model element, and it minted one per symbol. The
rule is gone from the tool. The rows it wrote are still in the journal, still
graded on every verify, and still counted as coverage.

This pass is the one that sorts them: what a person stood behind stays,
what the matcher guessed and nobody ever read is retired, and the cost of the
retirement is measured before it is paid.

Run it once per project. A journal that has already been through it reports
zero inferred rows and there is nothing to do.

### Ground rules

**The journal is append-only and this skill does not break that.** A retire is
an event like any other: the `add` stays, the `retire` lands on top, and the
fold shows the row gone. Nothing is rewritten and nothing is deleted from the
file. Never open `archi/links/journal.jsonl` in an editor — every move here is
a verb.

**Use the binary of the checkout you are in.** In a worktree, `./target/release/archi`.
The `archi` on `PATH` is often a symlink into a different checkout, and it will
answer about that project's journal instead of this one. Getting this wrong
produces a confident report about the wrong tree.

**Mutations need a worktree.** `archi worktree mint migrate-links`, then `cd`
there. The retire is a large, single, reviewable commit and it belongs on a
branch like any other work.

### 1. Read the shape before you touch anything

Every row carries the rule that produced it. The listing is
`id · kind · standing · origin · rule · <spec ref> ← <anchor>`, so the rule is
the fifth column:

```sh
archi link ls | awk '{print $5}' | sort | uniq -c
```

Three words appear. `authored` is a person typing `archi link add`. `declared`
is a task's writer naming its own work. `inferred` is the matcher. Only the
third is this pass's subject; the other two are somebody's claim and they
stay.

When `inferred` is zero, stop. There is nothing to migrate.

### 2. Measure the redundancy — this is the decision

The matcher minted one row per symbol, so one claim about one file arrives
dozens of times over. Count the distinct claims the inferred rows make against
the number of rows:

```sh
archi link ls | awk '$5=="inferred"' | wc -l
archi link ls | awk '$5=="inferred"' | sed 's/ ←.*//' \
  | awk '{$1=$2=$3=$4=$5=""; print}' | sed 's/^ *//' | sort -u | wc -l
```

A ratio near one means the matcher was accurate on this tree and the rows are
worth reading one by one. A ratio of ten or more means the corpus is one claim
repeated, and reading it row by row is not a review — it is the wall that
trains a reader to skim. On the project this skill was written from, 2141
inferred rows carried 102 distinct claims: twenty-one anchors each, and one
claim wore seventy-two.

Then measure what the retirement would cost, which is coverage — the count of
distinct spec refs any live link answers:

```sh
archi link ls | sed 's/ ←.*//' | awk '{$1=$2=$3=$4=$5=""; print}' \
  | sed 's/^ *//' | sort -u | wc -l
```

Write the number down. Run it again after step 4 and report both. On the
project this was written from it went 159 to 149: ten refs lost, and every one
of them was a ref whose only evidence was a shared word.

### 3. Read the claims, not the rows

Take the distinct claims from step 2 and read that list — tens of lines, not
thousands. For each one ask the only question that matters: **does this file
answer this part of the model?** You are not checking whether the anchor
resolves. `link verify` already knows that. You are checking whether the pair
was ever true.

A claim worth keeping is re-authored rather than rescued. Write it again:

```sh
archi link add "<spec ref>" <file>#<symbol> --kind indirect
```

That gives it one anchor chosen on purpose instead of twenty chosen by a word,
and the new row reads `authored`, which is what it now is. Do this before the
retire, so `link verify` never passes through a state where the claim is
missing.

**Put the ones you keep in the commit message.** A reader a year from now needs
to know which claims survived a mass retire and why, and the journal records
only that they were minted.

### 4. Retire the rest

Collect the ids and retire them in batches. `link rm` takes many ids at once
but a shell has an argument limit, and **zsh does not word-split an unquoted
variable**, so a naive `link rm $ids` sends the whole list as one id and fails
with a confusing message:

```sh
archi link ls | awk '$5=="inferred" {print $1}' > /tmp/inferred.txt
xargs -n 150 ./target/release/archi link rm < /tmp/inferred.txt
```

Re-run step 1. `inferred` is now zero, or holds only the rows you deliberately
kept.

### 5. Verify, and repair what the retire uncovered

```sh
archi link verify
archi link audit
```

Three things surface here and each has one move:

- **`missing`** — the anchor names a symbol that is not in the tree. The
  matcher anchored on test bodies and helper functions, and those get renamed.
  `archi link repin <id> --to <file>#<symbol>` when the code moved and the
  claim stands; `archi link rm <id>` when the symbol is gone for good.
- **`drifted`** — the body moved under a claim that still holds.
  `archi link repin <id>` re-pins it. Repin only what this pass touched. Rows
  that arrived drifted from somebody else's work are not yours to sign.
- **`unlinked spec element`** in the audit — a part of the model that now has
  no code against it. Some of these are the ten refs the retire cost. That is
  the honest state: the coverage was a word match, and it is better read as a
  gap than as an answer.

### 6. Commit as one unit, and report

One commit. The message carries the four numbers — rows retired, distinct
claims they expressed, coverage before, coverage after — and the list of
claims you re-authored by hand. Then land the worktree with the
`archi-finish-worktree` skill.

Report to the operator in those same numbers. "Retired 2141 rows expressing
102 claims; coverage 159 to 149" is a sentence they can argue with. "Cleaned up
the links" is not.

### Principles

- **The rule word is the whole triage.** `authored` and `declared` are
  somebody's claim. `inferred` is a guess nobody read.
- **Read claims, not rows.** The matcher's corpus is one claim repeated, and a
  row-by-row review of it teaches skimming.
- **Measure the cost before paying it.** Coverage before and after is the
  price of the retire, in one number.
- **Re-author what you keep.** A claim worth keeping deserves an anchor chosen
  on purpose, and the `authored` rule that says a person chose it.
- **Retiring appends.** Nothing is rewritten, nothing is lost, and the record
  of what the tool once believed stays readable.

## The scrap pass — sweep the waves an old binary left behind

A project planned before `the-plan-cleans-up-after-itself` carries
`waves/` folders its closes never deleted: today a successful close
removes the files its wave consumed, and completion removes `waves/`
whole. Under a completed plan the folder is dead weight from the older
binary, and this pass removes it once.

Confirm the plan's state first: `archi plan list` shows it completed,
and an old `plan.json` plan reads its state the same way. Then
`git rm -r archi/plans/<name>/waves` for each completed plan the
measurement named. A draft or started plan keeps its folder — its
files are what the next close reads. Close with `archi check` and one
commit naming the count of folders removed.

Deleting loses nothing. The record of what those waves accounted for
is the journal — every link carries its rule, its proving test and its
commit provenance — and git history keeps the snapshots.
