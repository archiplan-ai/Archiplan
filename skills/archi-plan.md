---
name: archi-plan
description: Generate an implementation plan from a hardened archi spec — a charter with a user-polled stack and its infrastructure, tasks per node, curated requirement ownership, named verifications, scenarios. This skill authors the plan only. The spec is /archi. The code is /archi-implement.
---

> **Skill freshness — the first step.** In an initialized project, run
> `archi sync-skills` before anything else. The report names
> `.claude/skills/archi-plan/SKILL.md`. When the act is `updated` or
> `created`, the text you follow is stale. Read that file again, follow
> it, and only then continue. `ok` means continue.

> **Working rules — they apply to every step of this session:**
> - **Bash output hygiene.** Do not print `echo` separators. Do not call
>   `python` or `jq` to reformat output that already reads well. Parse
>   output only when it genuinely narrows a large result to the slice you
>   need.
> - **User-facing output.** Keep the output friendly. Do not dump
>   archiplan jargon or internal element definitions into your messages.
>   Write plain, short summaries, and cite spec elements as inline code,
>   for example `BookingService`.

# Generate an implementation plan

You generate an implementation plan from a **hardened** archi spec: the
stress rounds survived and the version is saved. The plan is a folder of
markdown records under `archi/plans/<name>/`, and you author it like the
rest of the spec. Commands create and retire the files. You edit the prose
and the curation in them. `archi plan verify` lists the work to do. You
stop when `archi plan verify` is clean and the user has seen the plan.

Ask every question through the editor's poll tool (`AskUserQuestion` in
Claude Code, the equivalent elsewhere). Never write a freeform question
when the answer is a choice.

## Scope — what this skill does and does not do

This skill authors the **implementation plan**: the charter, the tasks,
the requirement ownership, the verifications and the scenarios. It does
**not** edit the spec, which is the work of `/archi`. It does **not**
write or edit application code or tests, which is the work of
`/archi-implement`. Its only write surface is the plan.

## Step 0 — The worktree (precondition)

You author the plan inside the worktree that carries its spec. The unit —
spec, then plan, then code — stays in one worktree and never in a primary
checkout. `archi status` answers where you are. Branch on it:

- **The session already works in a worktree**, and status shows a
  binding. Continue right there. An ongoing session never relocates.
- **A fresh session** does not pick a worktree itself. List the worktrees
  with `archi worktree ls`, and narrow with `--spec <effort>`. Ask the
  user through the poll tool **which worktree to work in**, then `cd`
  there. A worktree that exists only as a pushed branch re-attaches with
  `archi worktree mint <slug>`.
- **"not a git repository"** is a full stop. Follow the opening steps of
  the `archi` skill: create or cancel.

## Step 1 — Name and create the plan

Decide the name of the plan. When the user gave no name, ask through the
poll tool with two options: **automation**, where you derive a name from
the problem statement, and a **free-text field** for a name of their own.

Check whether a plan with that name exists, with `archi plan list`. When
it does, ask through the poll tool: **continue the existing plan**, or
**pick a different name**. Then run:

```
archi plan use <name>
```

It refuses on an unsaved model. Go back to `/archi` and run `version
save` first. A fresh name creates the record folder: the charter
`<name>.md`, `scenarios.md` and `state.json`. The folder is pinned to the
worktree's current spec version, and joined to its binding. Everything
below authors this plan. A plan in the old json form still reads and
still runs its lifecycle, but authoring refuses: `plan.json` is
read-only, because plans author as record folders now.

## Step 2 — Gather the full picture

```
archi query --top
archi check
archi version list
archi search <phrase> [--kind requirement|stressor|decision]
```

Build the mental model before you cut tasks: the ontology, the open
findings, the hardened version, and the decisions that priced the
trade-offs.

## Step 3 — Determine the task order

The dependency DAG lives in the `## Inputs` sections of the task files.
Every `- from t<N> — <note>` line records what flows in and a dependency
edge. The waves derive from it, and `plan verify` errors on an unknown
producer and on a cycle. Analyze the graph:

1. **Leaf nodes first** — the components with no dependencies.
2. **Data stores before services** — schema and storage before logic.
3. **Shared types and contracts before consumers** — interfaces before
   implementations.
4. **Bottom-up through the nesting** — child-scope components before
   parents.

## Step 4 — Author the plan

The folder is the plan. It holds the charter `<name>.md`, one
`t<N>-<node-slug>.md` per task, `scenarios.md`, and `state.json`.
`state.json` is lifecycle only. Commands move it, and you never edit it.
`plan use` creates the folder. `plan task add` creates a task file. `plan
task rm` retires one. You fill every slot inside the records by editing
the files, and `plan verify` (Step 5) lists the work that holds them
together.

Create the skeletons with the two commands, the folder once and a file per
task:

```
archi plan use <name>
archi plan task add <node> [--desc <text>]
```

A repeated create is safe. `task add` on a node whose file is still the
untouched skeleton reports "already minted". A file you have edited
refuses with "moved past its skeleton", because the command never overwrites
authored content. A wrong task retires with `plan task rm <id>`, in draft
only. Past draft, the message names `plan reset`. A task that other tasks
take as an input refuses and names its dependents: "feeds t3 — cut those
inputs first".

### The charter file

`archi/plans/<name>/<name>.md` holds the problem prose under the title,
before the first section. Then come two bullet sections:

```markdown
# tiny-store

a tiny hardened store

## Stack

- Rust — user choice
- <technology> — <where the choice came from: a user answer, a spec decision, a stressor's outcome>

## Architecture

- `<node>` — <one-line role>
- `<node>` realizes <which concrete tech realizes it>
```

A stack bullet holds the technology, then ` — `, then its provenance:
where the choice came from — a user answer, a spec decision, or the
outcome of a stressor. An architecture bullet opens with the backticked
node and takes one of two shapes. `— <role>` gives the one-line summary.
`realizes <tech>` gives the stack mapping. Cover **every top-level node**
with both shapes, because `plan verify` cross-checks the two in both
directions.

Derive the stack concerns from the node types: a runtime for `Service`,
an engine for `Storage`, and so on. Add the cross-cutting layer you
always ask about: **the test frameworks and libraries the user actually
uses**, for unit, integration and e2e tests. **Ask every choice through
the poll tool. Never assume.** One answered question does not license the
rest of the stack. When `mcp__context7__*` tools are exposed, consult
them before you offer the poll options. Read the current docs of the
candidate technologies **and of the test utilities**. Otherwise, rely on
what you know.

**Infrastructure.** Some products need running infrastructure: a
database, a queue, a browser for e2e tests, or provider emulators. For
those, recommend a configured docker setup — a compose file with the
utilities you judge right. Record it in the stack with its provenance,
and name the scenarios that depend on it. The goal is a working product
at the end, not code that never ran.

### Tasks — one file per node

`plan task add <node>` creates `t<N>-<node-slug>.md`. Its `## Spec`
section is seeded from the pinned model: the node plus its incoming
edges, in canonical form. You author everything else by editing the file:

```markdown
---
node: Store
owns: [store-encrypted]
---

# t1 — Store

persist rows

## Spec

- `Store`
- `<node_or_canonical_edge>`

## Inputs

- from <producer_task_id> — <concrete artifact that flows in>

## Outputs

- <relative/path>

## Stack

- <specific library / API / pattern / path>

## Verifications

### store-encrypted

- test — rows encrypted at rest
```

- `## Spec` — one backticked canonical ref per bullet. The seed is
  conservative. Add bullets for the outgoing edges the task realizes, the
  siblings it logically participates with, and the cross-scope edges it
  crosses. Each new ref widens the requirement-candidate set of the task.
- `## Inputs` — `- from t<N> — <note>`. The note names the concrete
  artifact that crosses the boundary: a schema, an interface, a DTO, a
  generated client, a migration. A weak note like "data from X" breaks
  the contract. When you cannot name what flows, the dependency probably
  should not exist.
- `## Outputs` — the files the task will write, as relative paths.
  Capture attributes deltas through them.
- `## Stack` — the task-level specifics: the library, the API, the
  pattern or the path.
- `## Verifications` — one `### <slug>` subhead per owned requirement,
  with proof bullets under it, as described below.

### Curate the requirements

The derived matched set is the candidate list. It is always fresh and
never retyped, and the reads are commands:

```
archi plan task req suggest <task_id>   # slot, slug, owned|candidate, via which refs
archi plan task req-list <task_id>      # the owned set with proof counts
```

Own a requirement by editing `owns: [...]` in the task's frontmatter.
Only matched candidates belong there. `plan verify` errors on an owned
slug that the reverse lookup does not match: drop the slug, or restore
the spec ref that carried it. **Curate the list. Do not accept it as it
comes.** One element can carry several requirements, and several tasks
can touch one element. To own everything everywhere duplicates work and
over-gates the verification. A task with candidates owns at least one,
and `plan verify` flags the opposite. A task whose elements carry no
requirements may own none. A missing candidate needs a spec ref, which is
a new `## Spec` bullet, never a hand-typed slug in `owns`. To un-own a
slug takes its proofs with it, so delete the `### <slug>` section too. A
verification under an unowned slug is structural, so own the slug first.

### Verifications

For every owned requirement, author at least one proof bullet under its
`### <slug>`. A proof bullet is a concrete, observable check that says
how the implementer will prove the claim: a failing test, a type
signature, a runtime contract, a migration assertion, or whatever the
prose of the requirement prescribes. **Do not paraphrase the requirement.
Name the check**, in the frameworks the user chose. That is why the
charter asked for them. A requirement that covers several distinct
concerns takes one bullet per concern, all under the same `### <slug>`.
An owned slug with no proof is a `plan verify` error.

### Task granularity

- A top-level node that is **not** decomposed takes one task.
- A decomposed node takes one task per child-scope node, **plus one
  integration task** for the parent.
- Shared types and contracts take one task.
- Data-store schemas take one task per store. Group them when they couple
  tightly.
- End-to-end coverage goes to the scenarios. They belong to the plan
  itself, not to a task.

### Scenarios — end-to-end user-story coverage

A user story crosses many elements, so to pin it to one element would lie
about its scope. Scenarios belong to the plan itself instead, in
`scenarios.md`: a heading, then one bullet per flow. `archi plan
scenarios list` reads them back.

```markdown
# Scenarios

- <one user-visible flow>
```

One flow is one bullet on one line. The record bullets do not wrap, here
or in the task files. To remove a scenario, delete its bullet.

Walk the architecture as a user, and enumerate every distinct
user-visible flow the product promises, one sentence each. Scenarios do
not link to requirements, and `plan verify` does not gate them. They are
the closing verification step of the implement stage. Name the
infrastructure that each scenario needs, which is the docker setup above.
The scenario step then has somewhere to run, instead of a silent skip.

## Step 5 — Verify and present

```
archi plan verify
```

The report lists all the work to do. It covers the structure, the unowned
candidates, the empty descriptions and the missing outputs. It covers the
verifications that are missing on owned requirements or keyed to unowned
ones. It covers the summary and mapping cross-check. Resolve every error.
Explain every note to the user. Then present the final plan:

```
archi plan show
archi plan task show <task_id>     # any brief the user wants to inspect
```

## Principles

- **The spec is the source of truth.** `task add` seeds the refs, and the
  reverse lookup suggests the candidates. Derive. Never retype.
- **Candidates are suggested. Ownership is curated.** `owns` is a strict
  subset of the matched set, and verify holds both directions.
- **Each task is a standalone brief.** `archi plan task show` renders
  everything a sub-agent needs. There is no implicit context.
- **The plan reflects the hardened architecture** — what survived the
  stress, not what was first proposed.
- **Verifications pull the work.** Each one is an observable check, named
  in the user's own frameworks. The implementation takes the shape that
  the check asks for.
- **Scenarios are the plan's own user stories, and they carry their
  infrastructure.**
- **Ask. Never assume.** Every stack and infrastructure choice goes
  through the poll tool.
- **Commands create and retire. Files carry the content.** Creation, removal
  and lifecycle go through the `archi plan` commands: `plan use`, `task
  add|rm`, `start`, `next` and `close`. Prose and curation are edits to
  the record files. `plan verify` lists the work that holds them
  together. `state.json` is lifecycle, so never hand-edit it.
