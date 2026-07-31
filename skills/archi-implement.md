---
name: archi-implement
description: Drive the implementation of a started archi plan — wave by wave, sub-agents in parallel, until `archi plan next` says `DONE`. Runs inside the worktree seat that carries the plan.
---

> **Skill freshness — the first move.** In an initialized project run
> `archi sync-skills` before anything else. If it reports
> `.claude/skills/archi-implement/SKILL.md` as `updated` (or `created`), the text
> you are following is stale: re-read that file, follow it, and only
> then continue. `ok` means proceed.

> **Working rules — apply to every step of this session:**
> - **Bash Output Hygiene.** No `echo` separators and no `python`/`jq` to reformat already-readable output. Parse only when it genuinely narrows large output to the slice you need.
> - **User-Facing Output.** Keep output user-friendly: don't dump archiplan jargon or internal element definitions into messages — write plain, concise summaries and cite spec elements as inline code (e.g. `Backend.api`).

# Implement an archi Plan

You are driving the implementation cycle of a plan authored via
`/archi-plan`. The plan is the source of truth — `archi plan
current-wave` names the tasks in flight, and the brief per task comes
from `archi plan task show <id>`: description, pinned version, spec_refs,
stack details, inputs, outputs, owned requirements with their
verification texts (unowned matches are other tasks' duty). The tool is
`archi`. You stop when `archi plan next` prints `DONE`.

Ask every question to the user through the editor's poll tool
(`AskUserQuestion` in Claude Code, the equivalent elsewhere) — never dump
a freeform question when the answer is a choice.

## Mandate — what this skill does and does not do

This is the skill that **writes code**: realize each plan task by editing
source and tests, then let capture seal the code-links. The boundary runs
the other way — do **not** redesign the architecture or reshape the spec
from the code. If implementation shows the spec is wrong, stop and take
it back to `/archi` (spec and stress stages); don't silently rewrite the
spec. Code is written against the plan's *pinned* version, never against
a moving spec.

## Step 0 — The seat (precondition)

Implementation runs inside the worktree that carries the plan — never in
a primary checkout. `archi status` answers where you are; branch on it:

- **The session already works in a worktree** (status shows a binding) →
  continue right there. An ongoing session never relocates.
- **A fresh session** does not pick a seat itself: list the seats —
  `archi worktree ls` (narrow with `--plan <name>`) — and ask the user
  through the poll tool **which worktree to work in**, then `cd` there. A
  seat existing only as a pushed branch re-attaches with
  `archi worktree mint <slug>`.
- **"not a git repository"** is a full stop (the `archi` skill's opening
  protocol: create-or-cancel).

Member code is edited only in the member worktree paths `archi status`
prints.

## Step 1 — Pick the plan

```
archi status
```

Its `open plans` listing is the menu. If it is empty, stop and send the
user to `/archi` (plan stage) — there is nothing to implement.

If an active plan is set here, ask through the poll tool: **implement
`<current_name>`**, **switch to a different plan**, **abort**. On
"switch", or if no plan is active, present the open plans through the
poll tool. Selecting one runs:

```
archi plan use <name>
```

## Step 2 — Synchronize lifecycle state

```
archi plan status
```

Branch on the printed state:

- `draft` → continue to Step 3.
- `started` → already running; **skip Step 3** and jump to Step 4.
- `completed` → ask through the poll tool: **reset and re-run**, **pick
  another plan**, **abort**. On reset, run `archi plan reset` and
  continue to Step 3.

## Step 3 — Verify and start

```
archi plan verify
archi plan start
```

If `verify` reports broken references or unverified requirements, surface
the CLI output verbatim and send the user to `/archi` (plan stage). If
`start` errors, surface the message verbatim — typical destinations: the
plan stage for an unready plan, the spec stages for spec-readiness,
Step 2 for a completed plan.

## Step 4 — Run waves

```
archi plan current-wave
archi plan task show <task_id>
```

For each task in the wave, read the brief verbatim, then dispatch one
sub-agent per task — **every task, a single-task wave included** — all in
one message so they run in parallel (see "Sub-agents" below). You never
implement a task inline: the orchestrator reads briefs, dispatches,
reviews, and runs the verbs.

Per-task contract (carried by the sub-agent's prompt):

a. **TDD.** Write failing tests derived from the task's owned-requirement
   verifications. Confirm red.
b. **context7.** If `mcp__context7__*` tools are exposed, query them for
   current docs of every library/framework the brief lists in
   `stack_details` — the test frameworks and utilities included.
   Otherwise rely on what you know.
c. Implement until tests are green — **inside the task's declared
   `outputs`**: capture attributes deltas through them, and code outside
   them lands as unaccounted.

Once every task in the wave is done, **commit the wave's work first** —
capture stamps every born link with the clean tree's commit as
provenance; a capture over a dirty tree births provenance-less links that
later audit as "no delta source". Then:

```
archi plan next
```

It captures the wave's delta into candidate links and gates on asserted
coverage of the refs the delta presses:

- blocked on coverage → not an error, the loop: the links are captured
  automatically — review `archi link ls --evidence`, `link confirm` the
  load-bearing candidates, `link rm` the drive-bys (subtractions stick),
  re-run `archi plan next`;
- prints the next wave → loop with it;
- prints the scenarios block → go to Step 5;
- prints `DONE` → stop.

`archi link verify` must be clean before the wave closes.

## Step 5 — Scenarios step

`plan next` printed the scenarios. Verify each end to end — one test per
scenario, on the infrastructure the plan named (its docker setup),
iterate until every scenario is green. **Missing infrastructure is a
stop, not a skip**: ask the user through the poll tool — stand the
infrastructure up now, or defer the scenario step with an explicit note
in your report; a silent `plan next` over unverified scenarios is
forbidden. Then:

```
archi plan next
```

It prints `DONE`. After the final commit, anchor the seat:
`archi version anchor` for the home repo, then
`archi version anchor --repo <member>` for every cascaded member the
seat carries — the landing gate refuses a stale mark, and anchoring at
DONE is what keeps the merge clean. After a squashed PR lands, the
anchor is what keeps baselines on real branches; skipping it is what
strands them. Then offer the user, through the
poll tool, to close the seat now — the `archi-finish-worktree` skill:
land the unit, push member branches, retire the worktree — or leave
the seat standing.

## Sub-agents

For every wave, dispatch one Agent call per task — a single-task wave
dispatches one — **all in a single message**, with
`subagent_type: general-purpose`. Each prompt must
be self-contained — sub-agents do not inherit conversation context.
Include the seat's working directory (and the member worktree path when
the task's outputs live in a member repo), the task id, its
`archi plan task show` brief verbatim, and the per-task contract (TDD,
context7 if available, implement inside the declared outputs). Sub-agents write code and tests
only — every `plan` and `link` verb stays with you, the orchestrator.
Wait for every sub-agent before you call `archi plan next`.

If a sub-agent will need a tool or permission the model has not yet been
granted, ask the user through the poll tool **before** dispatching —
sub-agents cannot prompt for permission on their own.

## Principles

- **Plan is the source of truth.** Read the in-flight ids through
  `archi plan current-wave` and the briefs through
  `archi plan task show`; do not improvise from memory.
- **Commit before the join.** The wave is committed before
  `archi plan next` — a link's commit provenance is stamped at birth,
  from a clean tree only.
- **TDD always.** Failing tests first; the brief's verifications are the
  contract.
- **Capture seals each wave.** A wave does not advance until every
  pressed ref is covered; confirm and prune the candidates, never skip
  the gate.
- **Every task runs in a sub-agent — no exceptions.** One Agent call per
  task, single-task waves included, one message, wait for all returns;
  the orchestrator never writes task code inline.
- **Stop on plan errors.** Surface the CLI message verbatim and route the
  user back to `/archi` as the message dictates.
- **CLI is the only author.** The plan is a record folder — never
  hand-edit its `state.json` lifecycle (`state`, `closed_waves`,
  latches), the version archive or the link journal least of all;
  authored plan fields were `/archi-plan`'s business, through its
  verbs, not this skill's.
- **Code lands only in seats.** The plan's worktree and its member
  worktrees — never a primary checkout.
