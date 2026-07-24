---
name: archi-plan
description: Generate an implementation plan from a hardened archi spec — an envelope with a user-polled stack and its infrastructure, tasks per node, curated requirement ownership, named verifications, scenarios. Authors the plan only; the spec is /archi, the code is /archi-implement.
---

> **Working rules — apply to every step of this session:**
> - **Bash Output Hygiene.** No `echo` separators and no `python`/`jq` to reformat already-readable output. Parse only when it genuinely narrows large output to the slice you need.
> - **User-Facing Output.** Keep output user-friendly: don't dump archiplan jargon or internal element definitions into messages — write plain, concise summaries and cite spec elements as inline code (e.g. `BookingService`).

# Generate Implementation Plan

You are generating an implementation plan from a **hardened** archi spec —
stress rounds survived, the version saved. The plan is authored entirely
through `archi plan` subcommands; the file behind them stays text and
merges like text, but your pen is the CLI. You stop when
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
A fresh name pins the seat's current spec version and joins the seat's
binding; everything below authors this plan.

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

The dependency DAG is authored through `archi plan task input add …
--from <producer>` — every `--from` records both "what flows in" and a
dependency edge; the waves derive from it, and
`plan verify` errors on unknown producers and cycles. Analyze the graph:

1. **Leaf nodes first** — components with no dependencies.
2. **Data stores before services** — schema/storage before logic.
3. **Shared types/contracts before consumers** — interfaces before
   implementations.
4. **Bottom-up through nesting** — child-scope components before parents.

## Step 4 — Author the plan

**Batch the authoring.** `archi batch -` executes commands from stdin —
one per line, `#` comments and blank lines skipped, `\n` inside double
quotes carries a multiline value, fail-fast with the offending line
named. Author the whole envelope, tasks and curation in one call instead
of a verb per turn:

```
archi batch - <<'EOF'
plan problem "a tiny hardened store"
plan tech add Rust --provenance "user choice"
plan task add Store --desc "persist rows"
plan task req add t1 store-encrypted
plan task verification add t1 store-encrypted "test — rows encrypted at rest"
EOF
```

### Envelope — the plan-level frame

Author once, before any task:

```
archi plan problem "<what this plan delivers, in product terms>"
archi plan tech add "<technology>" --provenance "<where the choice came from: a user answer, a spec decision, a stressor's outcome>"
archi plan architecture-summary add <node> "<one-line role>"    # one per top-level node
archi plan stack-mapping add <node> "<which concrete tech realizes it>"
```

Cover **every top-level node** with both a summary and a mapping
entry — `plan verify` cross-checks the two both ways.

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

### Tasks — one per node

```
archi plan task add <node> [--desc <text>]
```

The auto-seed is conservative: the node plus its incoming edges, in
canonical form. Outgoing edges the task realizes, siblings it logically
participates with, cross-scope edges it crosses — add them explicitly;
each new ref widens the task's requirement-candidate set:

```
archi plan task spec-ref add <task_id> <node_or_canonical_edge>
```

Then fill in what the spec cannot provide:

```
archi plan task desc <task_id> '<markdown body>'
archi plan task stack-detail add <task_id> "<specific library / API / pattern / path>"
archi plan task input add <task_id> "<concrete artifact that flows in>" --from <producer_task_id>
archi plan task output add <task_id> <relative/path>
```

The **input note** names the concrete artifact crossing the boundary
(schema, interface, DTO, generated client, migration…) — weak notes
("data from X") break the contract: if you can't name what flows, the
dependency probably shouldn't exist. Outputs are the files the task will
write — capture attributes deltas through them.

### Curate requirements

The derived matched set is the candidate list — always fresh, never
retyped. List the candidates, then own the ones the task answers for:

```
archi plan task req suggest <task_id>          # slot, slug, owned|candidate, via which refs
archi plan task req add <task_id> <slug|slot>  # only a matched candidate is accepted
archi plan task req remove <task_id> <slug>    # takes its verifications with it
archi plan task req-list <task_id>             # the owned set with proof counts
```

**Curate, don't rubber-stamp**: one element can carry several
requirements and several tasks can touch one element; owning everything
everywhere duplicates work and over-gates verification. A task with
candidates owns at least one — `plan verify` flags the opposite; a task
whose elements carry no requirements may legitimately own none. A missing
candidate wants a spec_ref (`task spec-ref add`), never a hand-typed slug.

### Verifications

For every owned requirement, author at least one verification — a
concrete, observable check describing how the implementer will prove the
claim: a failing test, a type signature, a runtime contract, a migration
assertion — whatever the requirement's prose prescribes:

```
archi plan task verification add <task_id> <slug|slot> "<observable check>"
```

**Do not paraphrase the requirement — name the check**, in the frameworks
the user chose (that is why the envelope asked). A requirement covering
several distinct concerns takes one verification per concern — same slot,
several `verification add` calls.

### Task granularity

- Top-level nodes **not** decomposed → one task each.
- Decomposed → one task per child-scope node **plus one integration
  task** for the parent.
- Shared types/contracts → one task.
- Data-store schemas → one task per store (grouped if tightly coupled).
- End-to-end coverage → scenarios (envelope data, not tasks).

### Scenarios — end-to-end user-story coverage

A user story crosses many elements; pinning it to one would lie about its
scope, so scenarios live on the plan envelope as free text:

```
archi plan scenarios add "<one user-visible flow>"
archi plan scenarios list
archi plan scenarios remove <index>            # 1-based, from `list`
```

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
- **CLI is the only author.** Every mutation goes through `archi plan …`;
  every read through `plan show` / `task show` / `req suggest` /
  `req-list`. Do not hand-write plan.json or poke at lifecycle state.
