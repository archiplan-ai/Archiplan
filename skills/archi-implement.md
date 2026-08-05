---
name: archi-implement
description: Drive the implementation of a started archi plan — wave by wave, sub-agents in parallel, until `archi plan next` says `DONE`. Runs inside the worktree that carries the plan.
---

> **Skill freshness — the first step.** In an initialized project, run
> `archi sync-skills` before anything else. The report names
> `.claude/skills/archi-implement/SKILL.md`. When the act is `updated` or
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
>   for example `Backend.api`.

# Implement an archi plan

You drive the implementation cycle of a plan authored through
`/archi-plan`. The plan is the source of truth. `archi plan current-wave`
names the tasks in flight. `archi plan task show <id>` gives the brief
per task: the description, the pinned version, the spec_refs, the stack
details, the inputs, the outputs, and the owned requirements with their
verification texts. Unowned matches are the duty of other tasks. The tool
is `archi`. You stop when `archi plan next` prints `DONE`.

Ask every question through the editor's poll tool (`AskUserQuestion` in
Claude Code, the equivalent elsewhere). Never write a freeform question
when the answer is a choice.

## Scope — what this skill does and does not do

This is the skill that **writes code**. Realize each plan task by editing
source and tests, then let capture seal the code-links. The boundary runs
the other way: do **not** redesign the architecture, and do not reshape
the spec from the code. When the implementation shows that the spec is
wrong, stop and take it back to `/archi`, to the spec and stress stages.
Never rewrite the spec silently. You write code against the *pinned*
version of the plan, never against a moving spec.

## Step 0 — The worktree (precondition)

The implementation runs inside the worktree that carries the plan, never
in a primary checkout. `archi status` answers where you are. Branch on
it:

- **The session already works in a worktree**, and status shows a
  binding. Continue right there. An ongoing session never relocates.
- **A fresh session** does not pick a worktree itself. List the worktrees
  with `archi worktree ls`, and narrow with `--plan <name>`. Ask the user
  through the poll tool **which worktree to work in**, then `cd` there. A
  worktree that exists only as a pushed branch re-attaches with `archi
  worktree mint <slug>`.
- **"not a git repository"** is a full stop. Follow the opening steps of
  the `archi` skill: create or cancel.

Edit member code only in the member worktree paths that `archi status`
prints.

Before the first wave, when the ground is ambiguous — more than one
standing worktree, or work that may depend on another unit — confirm it
through the poll tool: stay in this worktree and the member worktrees
`archi status` prints, or attach another one. A task that depends on
unlanded work standing elsewhere is a question to the user, never your
own call. Branches come only from the mint cascade. Never run a raw
`git checkout`, `git switch` or `git checkout -b` yourself, in the home
worktree or in a member worktree. The worktrees already stand on the
branches of the unit. When the ground looks wrong, that is the poll
above, not a checkout.

## Step 1 — Pick the plan

```
archi status
```

Its `open plans` listing is the menu. When it is empty, stop and send the
user to `/archi`, to the plan stage. There is nothing to implement.

When an active plan is set here, ask through the poll tool with three
options: **implement `<current_name>`**, **switch to a different plan**,
or **abort**. On a switch, or when no plan is active, present the open
plans through the poll tool. A selection runs:

```
archi plan use <name>
```

## Step 2 — Synchronize the lifecycle state

```
archi plan status
```

Branch on the printed state:

- `draft` — continue to Step 3.
- `started` — the plan already runs. **Skip Step 3** and go to Step 4.
- `completed` — ask through the poll tool with three options: **reset and
  re-run**, **pick another plan**, or **abort**. On a reset, run `archi
  plan reset` and continue to Step 3.

## Step 3 — Verify and start

```
archi plan verify
archi plan start
```

When `verify` reports broken references or unverified requirements,
surface the CLI output verbatim and send the user to `/archi`, to the
plan stage. When `start` errors, surface the message verbatim. The
typical destinations are the plan stage for an unready plan, the spec
stages for spec readiness, and Step 2 for a completed plan.

## Step 4 — Run the waves

```
archi plan current-wave
archi plan task show <task_id>
```

For each task in the wave, read the brief verbatim. Then dispatch one
sub-agent per task — **every task, a single-task wave included** — all in
one message, so they run in parallel. See "Sub-agents" below. You never
implement a task inline. The orchestrator reads the briefs, dispatches
the sub-agents, reviews the work, and runs the commands.

The per-task contract, carried by the prompt of the sub-agent:

a. **TDD.** Write failing tests derived from the owned-requirement
   verifications of the task. Confirm red.
b. **context7.** When `mcp__context7__*` tools are exposed, query them
   for current docs. Cover every library and framework that the brief
   lists in `stack_details`, the test frameworks and utilities included.
   Otherwise, rely on what you know.
c. Implement until the tests are green, **inside the declared `outputs`
   of the task**. Capture attributes deltas through them, and code
   outside them lands as unaccounted.

When every task in the wave is done, **commit the work of the wave
first**. Capture stamps every new link with the commit of the clean tree
as its provenance. A capture over a dirty tree creates links with no
provenance, and a later audit reports them as "no delta source". Then
run:

```
archi plan next
```

It captures the delta of the wave into candidate links, and it gates on
the asserted coverage of the refs that the delta presses:

- Blocked on coverage. This is not an error. It is the loop. The links
  are captured automatically, so review `archi link ls --evidence`. Run
  `link confirm` on the load-bearing candidates, and `link rm` on the
  incidental ones. A removal sticks. Then run `archi plan next` again.
- It prints the next wave. Loop with it.
- It prints the scenarios block. Go to Step 5.
- It prints `DONE`. Stop.

`archi link verify` must be clean before the wave closes.

## Step 5 — The scenarios step

`plan next` printed the scenarios. Verify each one end to end: one test
per scenario, on the infrastructure the plan named, which is its docker
setup. Iterate until every scenario is green. **Missing infrastructure is
a stop, not a skip.** Ask the user through the poll tool: start the
infrastructure now, or defer the scenario step with an explicit note in
your report. A silent `plan next` over unverified scenarios is forbidden.
Then run:

```
archi plan next
```

It prints `DONE`. After the final commit, anchor the worktree. Run `archi
version anchor` for the home repo, then `archi version anchor --repo
<member>` for every member the worktree carries. The landing gate refuses
a stale mark, and to anchor at DONE is what keeps the merge clean. After
a squashed PR lands, the anchor is what keeps the baselines on real
branches. To skip it is what strands them. Then put one question to the
user through the poll tool, with two options. **Close the worktree now**
with the `archi-finish-worktree` skill: land the unit, push the member
branches, retire the worktree. Or **leave the worktree standing**.

## Sub-agents

For every wave, dispatch one Agent call per task. A single-task wave
dispatches one call. Send them **all in a single message**, with
`subagent_type: general-purpose`. Every prompt must be self-contained,
because sub-agents do not inherit the conversation context. Include the
working directory of the worktree, and the member worktree path when the
outputs of the task live in a member repo. Include the task id, its
`archi plan task show` brief verbatim, and the per-task contract: TDD,
context7 when available, and implementation inside the declared outputs.
Every sub-agent prompt forbids branch creation and branch switching —
sub-agents write code on the branches the worktrees already stand on.
Sub-agents write code and tests only. Every `plan` and `link` command stays
with you, the orchestrator. Wait for every sub-agent before you call
`archi plan next`.

A sub-agent can need a tool or a permission that the model does not yet
have. Ask the user through the poll tool **before** you dispatch.
Sub-agents cannot prompt for permission on their own.

## Principles

- **The plan is the source of truth.** Read the in-flight ids through
  `archi plan current-wave` and the briefs through `archi plan task
  show`. Do not improvise from memory.
- **Commit before the merge.** Commit the wave before `archi plan next`.
  The commit provenance of a link is stamped when the link is created,
  and only from a clean tree.
- **TDD always.** Failing tests come first, and the verifications in the
  brief are the contract.
- **Capture seals each wave.** A wave does not advance until every
  pressed ref is covered. Confirm and prune the candidates. Never skip
  the gate.
- **Every task runs in a sub-agent. There are no exceptions.** One Agent
  call per task, single-task waves included, all in one message, and you
  wait for every return. The orchestrator never writes task code inline.
- **Stop on plan errors.** Surface the CLI message verbatim, and route
  the user back to `/archi` as the message dictates.
- **The CLI is the only author.** The plan is a record folder. Never
  hand-edit its `state.json` lifecycle (`state`, `closed_waves`,
  latches), and least of all the version archive or the link journal. The
  authored plan fields were the business of `/archi-plan`, through its
  commands, not of this skill.
- **No raw checkouts.** The orchestrator never creates or switches a
  branch by hand. The mint cascade is the only branch maker, and the
  worktrees stand on the branches of the unit.
- **Code lands only in worktrees** — the worktree of the plan and its
  member worktrees, never a primary checkout.
