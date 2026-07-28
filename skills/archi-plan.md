---
name: archi-plan
description: Generate an implementation plan from a hardened archi spec — an envelope with a user-polled stack and its infrastructure, tasks per node, curated requirement ownership, named verifications, scenarios. Authors the plan only; the spec is /archi, the code is /archi-implement.
---

> **Skill freshness — the first move.** In an initialized project run
> `archi sync-skills` before anything else. If it reports
> `.claude/skills/archi-plan/SKILL.md` as `updated` (or `created`), the text
> you are following is stale: re-read that file, follow it, and only
> then continue. `ok` means proceed.

> **Working rules — apply to every step of this session:**
> - **Bash Output Hygiene.** No `echo` separators and no `python`/`jq` to reformat already-readable output. Parse only when it genuinely narrows large output to the slice you need.
> - **User-Facing Output.** Keep output user-friendly: don't dump archiplan jargon or internal element definitions into messages — write plain, concise summaries and cite spec elements as inline code (e.g. `BookingService`).

# Generate Implementation Plan

You are generating an implementation plan from a **hardened** archi spec —
stress rounds survived, the version saved. The plan is a folder of
markdown records under `archi/plans/<name>/`, authored like the rest of
the spec: verbs mint and retire the files, prose and curation are edited
in them, and `archi plan verify` is the worklist. You stop when
`archi plan verify` is clean and the user has seen the plan.

Ask every question to the user through the editor's poll tool
(`AskUserQuestion` in Claude Code, the equivalent elsewhere) — never dump
a freeform question when the answer is a choice.

## Mandate — what this skill does and does not do

This skill authors the **implementation plan**: envelope, tasks,
requirement ownership, verifications, scenarios. It does **not** edit the
spec (that's `/archi`) and does **not** write or edit application code or
tests (that's `/archi-implement`). Its only write surface is the plan.

## Step 0 — The seat (precondition)

The plan is authored inside the worktree seat that carries its spec — the
unit (spec → plan → code) rides one seat, never a primary checkout.
`archi status` answers where you are; branch on it:

- **The session already works in a worktree** (status shows a binding) →
  continue right there. An ongoing session never relocates.
- **A fresh session** does not pick a seat itself: list the seats —
  `archi worktree ls` (narrow with `--spec <effort>`) — and ask the user
  through the poll tool **which worktree to work in**, then `cd` there. A
  seat existing only as a pushed branch re-attaches with
  `archi worktree mint <slug>`.
- **"not a git repository"** is a full stop (the `archi` skill's opening
  protocol: create-or-cancel).

## Step 1 — Name and create the plan

Decide the plan's name. If the user did not provide one, ask through the
poll tool with two options: **automation** (you derive a name from the
problem statement) and a **free-text field** for their own.

Check whether a plan with that name already exists (`archi plan list`).
If it does, ask through the poll tool: **continue the existing plan** or
**pick a different name**. Then:

```
archi plan use <name>
```

It refuses on an unsaved model — back to `/archi`, `version save` first.
A fresh name mints the record folder — charter `<name>.md`,
`scenarios.md`, `state.json` — pinned to the seat's current spec version
and joined to the seat's binding; everything below authors this plan. A
plan in the old json form still reads and runs its lifecycle, but
authoring refuses: legacy plan.json is read-only — plans author as
record folders now.

## Step 2 — Gather the full picture

```
archi query --top
archi check
archi version list
archi search <phrase> [--kind requirement|stressor|decision]
```

Build the mental model — the ontology, the open findings, the hardened
version, the decisions that priced the trade-offs — before cutting tasks.

## Step 3 — Determine task ordering

The dependency DAG lives in the task files' `## Inputs` sections — every
`- from t<N> — <note>` line records both "what flows in" and a
dependency edge; the waves derive from it, and `plan verify` errors on
unknown producers and cycles. Analyze the graph:

1. **Leaf nodes first** — components with no dependencies.
2. **Data stores before services** — schema/storage before logic.
3. **Shared types/contracts before consumers** — interfaces before
   implementations.
4. **Bottom-up through nesting** — child-scope components before parents.

## Step 4 — Author the plan

The folder is the plan: the charter `<name>.md` (envelope), one
`t<N>-<node-slug>.md` per task, `scenarios.md`, and `state.json` —
lifecycle only, moved by verbs, never edited. Two verbs mint and retire
the task files; every slot inside the records is filled by editing the
files, and `plan verify` (Step 5) is the worklist that holds them
together.

**Batch the minting.** `archi batch -` executes commands from stdin —
one per line, `#` comments and blank lines skipped, fail-fast with the
offending line named. Mint the whole skeleton set in one call:

```
archi batch - <<'EOF'
plan use tiny-store
plan task add Types --desc "the shared Row schema"
plan task add Store --desc "persist rows encrypted"
plan task add API --desc "the read/write surface"
EOF
```

Mints converge: `task add` on a node whose file is still the untouched
skeleton reports "already minted"; a file you have edited refuses —
"moved past its skeleton" — the verb never overwrites authored content.
A mis-cut task retires with `plan task rm <id>` — draft only (past
draft it names `plan reset`), and a task other tasks input refuses with
its dependents: "feeds t3 — cut those inputs first".

### Envelope — the charter file

`archi/plans/<name>/<name>.md`: the problem prose sits under the title,
before the first section; then two bullet sections —

```markdown
# tiny-store

A tiny hardened store: rows persist encrypted, served over HTTP.

## Stack

- Rust — user choice
- SQLCipher — decision `encrypt-at-rest`
- cargo-nextest — the user's test runner

## Architecture

- `Store` — persists and encrypts the rows
- `API` — the read/write surface
- `Store` realizes SQLCipher
- `API` realizes Rust (axum)
```

A stack bullet is the technology, ` — `, and its provenance — where the
choice came from: a user answer, a spec decision, a stressor's outcome.
Architecture bullets open with the backticked node and take two shapes:
`— <role>` (the one-line summary) and `realizes <tech>` (the stack
mapping). Cover **every top-level node** with both — `plan verify`
cross-checks the two both ways.

Derive stack concerns from the node types (runtime for `Service`, engine
for `Storage`, …) plus the always-ask cross-cutting layer: **the test
frameworks and libraries the user actually uses** (unit, integration,
e2e). **Ask every choice through the poll tool — never assume**: one
answered question does not license the rest of the stack. If
`mcp__context7__*` tools are exposed, consult them for current docs of
the candidate technologies **and the test utilities** before offering
poll options; otherwise rely on what you know.

**Infrastructure.** When the product needs running infrastructure — a
database, a queue, a browser for e2e, provider emulators — recommend a
configured docker setup (a compose file with the utilities you judge
right), record it in the stack with its provenance, and name which
scenarios depend on it. The goal is a working product at the end, not
code that never ran.

### Tasks — one file per node

`plan task add <node>` mints `t<N>-<node-slug>.md`, its `## Spec` seeded
from the pinned model — the node plus its incoming edges, in canonical
form. Everything else is authored by editing the file:

```markdown
---
node: API
owns: [rows-served]
---

# t3 — API

The HTTP surface over the store: one handler per verb.

## Spec

- `API`
- `UI.send request(Row) API.write`

## Inputs

- from t1 — the Row schema and the storage trait
- from t2 — the Store constructor the router holds

## Outputs

- src/api.rs

## Stack

- axum 0.8: one router, state = the Store handle

## Verifications

### rows-served

- test — POST then GET round-trips a row (axum-test)
```

- `## Spec` — one backticked canonical ref per bullet. The seed is
  conservative: outgoing edges the task realizes, siblings it logically
  participates with, cross-scope edges it crosses — add their bullets;
  each new ref widens the task's requirement-candidate set.
- `## Inputs` — `- from t<N> — <note>`; the note names the concrete
  artifact crossing the boundary (schema, interface, DTO, generated
  client, migration…) — weak notes ("data from X") break the contract:
  if you can't name what flows, the dependency probably shouldn't exist.
- `## Outputs` — the files the task will write, as relative paths —
  capture attributes deltas through them.
- `## Stack` — task-level specifics: the library, API, pattern or path.
- `## Verifications` — one `### <slug>` subhead per owned requirement,
  proof bullets under it (below).

### Curate requirements

The derived matched set is the candidate list — always fresh, never
retyped; the reads stay verbs:

```
archi plan task req suggest <task_id>   # slot, slug, owned|candidate, via which refs
archi plan task req-list <task_id>      # the owned set with proof counts
```

Own by editing `owns: [...]` in the task's frontmatter — only matched
candidates belong there; `plan verify` errors on an owned slug the
reverse lookup does not match (drop it, or restore the spec ref that
carried it). **Curate, don't rubber-stamp**: one element can carry
several requirements and several tasks can touch one element; owning
everything everywhere duplicates work and over-gates verification. A
task with candidates owns at least one — `plan verify` flags the
opposite; a task whose elements carry no requirements may legitimately
own none. A missing candidate wants a spec ref (a new `## Spec` bullet),
never a hand-typed slug in `owns`. Un-owning a slug takes its proofs
with it: delete the `### <slug>` section too — a verification under an
unowned slug is structural, own it first.

### Verifications

For every owned requirement, author under its `### <slug>` at least one
proof bullet — a concrete, observable check describing how the
implementer will prove the claim: a failing test, a type signature, a
runtime contract, a migration assertion — whatever the requirement's
prose prescribes. **Do not paraphrase the requirement — name the
check**, in the frameworks the user chose (that is why the envelope
asked). A requirement covering several distinct concerns takes one
bullet per concern, all under the same `### <slug>`. An owned slug with
no proof is a `plan verify` error.

### Task granularity

- Top-level nodes **not** decomposed → one task each.
- Decomposed → one task per child-scope node **plus one integration
  task** for the parent.
- Shared types/contracts → one task.
- Data-store schemas → one task per store (grouped if tightly coupled).
- End-to-end coverage → scenarios (envelope data, not tasks).

### Scenarios — end-to-end user-story coverage

A user story crosses many elements; pinning it to one would lie about
its scope, so scenarios live on the plan envelope — `scenarios.md`, a
heading and one bullet per flow (`archi plan scenarios list` reads them
back):

```markdown
# Scenarios

- A user writes a row over HTTP and reads it back decrypted — needs the compose db service.
```

One flow is one bullet on one line — the record bullets (here and in the
task files) do not wrap.

Walk the architecture as a user and enumerate every distinct user-visible
flow the product promises, one sentence each. They are not linked to
requirements and `plan verify` does not gate them — but they are the
closing verification step of the implement stage: name inside each the
infrastructure it needs (the docker setup above), so the scenario step
has somewhere to run instead of a silent skip.

## Step 5 — Verify and present

```
archi plan verify
```

The report is the whole worklist: structure, unowned candidates,
verifications missing on owned requirements or keyed to unowned ones,
empty descriptions, missing outputs, the summary/mapping cross-check.
Resolve every error; explain every note to the user. Present the final
plan:

```
archi plan show
archi plan task show <task_id>     # any brief the user wants to inspect
```

## Principles

- **The spec is the source of truth.** `task add` seeds the refs, the
  reverse lookup suggests the candidates — derive, never retype.
- **Candidates are suggested, ownership is curated.** `owns` is a strict
  subset of the matched set; verify holds both directions.
- **Each task is a standalone brief.** `archi plan task show` renders
  everything a sub-agent needs — no implicit context.
- **The plan reflects the hardened architecture** — what survived stress,
  not what was first proposed.
- **Verifications pull the work.** Each is an observable check named in
  the user's own frameworks; implementation takes the shape the check
  asks for.
- **Scenarios are envelope user stories carrying their infrastructure.**
- **Ask, never assume.** Every stack and infrastructure choice goes
  through the poll tool.
- **Verbs cut, files carry.** Creation, removal and lifecycle go through
  `archi plan` verbs (`plan use`, `task add|rm`, `start`/`next`/`close`);
  prose and curation are edits to the record files; `plan verify` is the
  worklist that holds them together. `state.json` is lifecycle — never
  hand-edit it.
