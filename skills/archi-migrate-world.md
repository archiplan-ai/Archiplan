---
name: archi-migrate-world
description: Give a standing archiplan project its world facts — read the world prose its intents already hold and the story blocks its old plans authored, interview one candidate at a time, and write a fact only where a person can name the workaround. Use when a project modeled before the wing must record the outside conditions its spec assumes.
---

> **Skill freshness — the first step.** In an initialized project, run
> `archi sync-skills` before anything else. The report names
> `.claude/skills/archi-migrate-world/SKILL.md`. When the act is
> `updated` or `created`, the text you follow is stale. Read that file
> again, follow it, and only then continue. `ok` means continue.

# Migrate a standing project into the wing

A project modeled before the wing already claims things about the world.
The claims sit in the opening prose of its intents, and in the free-text
story blocks its old plans authored. No verb reads them there, and
nothing in the repository can make them false. This skill moves the ones
a person can stand behind into `archi/world/`, one at a time, by
interview.

It deletes nothing. The intent keeps its paragraph, the old plan keeps
its stories, and the wing stands beside them. The run returns two
things: the facts that landed, and a brief of what did not map and why.

Judgement is the whole job here, and judgement is why this is a skill
and not a converter. Naming the condition under a story is a person's
call. A converter would have written fluent facts nobody observed, which
is worse than an empty wing, because an invented fact reads exactly like
an observed one.

## 1. Read what the project already claims

Run these in the project. They give the candidate list and what already
stands:

```sh
archi world ls                       # the wing today; often empty
archi search <phrase> --kind intent  # the world prose, by phrase
ls archi/requirements/*/             # the intents, one folder each
ls archi/plans/*/scenarios.md        # the story blocks, if the tree has old plans
```

Two places hold candidates, and nothing else does:

- **The intents.** The opening paragraph of
  `archi/requirements/<intent>/<intent>.md`. A sentence about people,
  their days, their machines or their money is a candidate. A sentence
  about what the system must do is not: that is a requirement and it
  already has a home.
- **The old plans.** `archi/plans/<name>/scenarios.md`, written before a
  plan was forbidden to author stories. No verb reads those blocks. Each
  story that rests on an unstated condition is a candidate.

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
3. **The killer.** What would you have to see to call the condition
   over?
4. **The behavior.** Which scenarios does the condition dictate? One is
   enough to start.
5. **The reach.** Which parts of the model does the story touch?

**The gate is question 2.** When the operator cannot name a workaround,
there is no condition — only a wish — and this skill writes nothing for
that candidate. It goes in the brief with the sentence it came from, and
you move to the next one. Do not soften the question, do not answer it
from the prose, and do not mint a skeleton "to fill in later". An empty
wing is an honest state. A fact nobody observed is not.

## 3. Write the fact

One candidate that passed the gate is one file:

```sh
archi world add "<the fact in one line>"
```

The verb mints `archi/world/<slug>.md` with the three lists empty and
the headings in place. You write the prose:

- **The name** is the fact in one line, as the operator said it.
- **The paragraph under it** states what condition this is and why the
  behavior follows from it.
- **`## What kills this`** is the observation that would end the fact.
- **`## Scenarios`** holds one `Feature:` and its scenarios. The grammar
  is `Feature`, `Scenario`, `Given`, `When`, `Then`, `And`, and a tag
  line; `@runs:<member>` names the member whose tree runs the scenario.

Write the fact **without the nouns of the model**. The condition is
about the world, so the world's words are the right ones, and `check`
reports `world_speaks_the_model(<element>)` when a model name leaks into
the name or the paragraph.

The header points three ways:

- **`covers`** takes the elements the story touched, by absolute path.
  Leave it empty when the model has not reached the fact. `check`
  reports `world_uncovered` for that, and it is early work, not a
  defect.
- **`sources`** names the file the claim came from — the intent, or the
  plan whose story carried it — as a path from the project root. The
  entry resolves, so the fact is grounded and a migrated project reports
  nothing. Say plainly what this records: provenance, not observation.
  The ground under a migrated fact is prose in this tree, and the reader
  sees that at a glance. When the operator names material outside the
  tree — a ticket, an interview, a measurement — add its locator beside
  the file.
- **`uses`** names another fact this one holds only while that one
  holds. Empty is the normal case.

Then leave the origin alone. Never delete the intent's paragraph, never
delete or trim `scenarios.md`, and never edit either to point at the new
file. The claim now stands in two places, and that is the accepted
price.

## 4. Check, then hand back the brief

Run `archi check` and report what it said, verbatim in substance:

- Errors block. An unresolved `covers` entry, an unresolved `sources`
  path or a scenario outside the grammar is a fact that is not finished.
  Fix it now.
- The closing line counts the wing: `world — <n> facts · <m>
  ungrounded`. A migration leaves `m` at what it was, because every fact
  it wrote names its origin file.
- Findings are the worklist, never a reason to stop.

Read the wing back the way a reader will: `archi world ls`, and `archi
world ls --covers <element>` from a node to the conditions that rule it.

Then write `world-migration-brief.md` at the project root and tell the
operator it is there. It holds:

- **What landed** — one line per fact: the slug, the file it came from,
  and the workaround the operator named.
- **What did not map, and why** — every candidate that stopped at the
  gate, each with its sentence and the file it sits in. Name the reason
  in the operator's terms, not as a verdict.
- **What is thin** — facts with one scenario, facts with an empty
  `covers`, and conditions the operator named but could not date.
- **Where the prose still stands** — the intents and the `scenarios.md`
  files this run read and left untouched.

A later run starts from that brief. The candidates that stopped at the
gate are the ones to ask about again, when somebody has watched the
work.
