---
name: archi-resume
description: One door into work that already stands — read the seats, the binding and the plan lifecycles from the record, route each state to the workflow skill that continues it, then enter the chosen worktree. Use when a session picks standing work up — after a clear, a restart or a handover — before any workflow skill opens.
---

> **Skill freshness — the first step.** In an initialized project, run
> `archi sync-skills` before anything else. The report names
> `.claude/skills/archi-resume/SKILL.md`. When the act is `updated` or
> `created`, the text you follow is stale. Read that file again, follow
> it, and only then continue. `ok` means continue.

> Retrieval — how to find anything here — is the `archi-search` skill.

# Archi resume — one door into the standing work

A fresh session holds none of what the last one knew: which seats
stand, in what state, and which skill continues each. The record
answers all three, so this page is three parts — the reads, the
routing table, the seat entry. The page chooses and re-teaches
nothing: every workflow skill keeps its own opening check, and the
chosen skill runs it after the choice.

## The reads

The record before archaeology, in this order:

1. `archi worktree ls [--status active|closed|all] [--plan <slug>]
   [--spec <effort>]` — the seats, one row each: path, branch, plan,
   spec. A `waiting` row is work in flight and is never closed.
2. `archi status` — this checkout's binding, the plan state, the
   version state, the open stress round, and the member worktree
   paths.
3. `archi repo ls` — on a cascaded seat, each member's health.
4. `archi plan list` — every plan with its lifecycle.
5. For anything the question names, `archi version list` and the
   `archi-search` page.

Git history is the last resort, and reading it first is what
produces a report about the wrong round.

## The routing table

State to skill:

- plan `draft` — finish authoring in `archi-plan`.
- plan `started` — `archi-implement` picks the wave up.
- plan `completed` with the seat standing — land through
  `archi-finish-worktree`, or continue a new unit in the same
  binding.
- an open stress round or an unsaved model — the `archi` skill, at
  that stage.
- a worktree that exists only as a pushed branch — re-attach with
  `archi worktree mint <slug>`; it attaches, never creates. Then
  route by its plan state.
- nothing standing at all — new work through `archi`.

## The seat entry

`cd` into the chosen worktree yourself: the CLI never changes your
directory. More than one standing seat is one question through the
poll tool (AskUserQuestion), the seats as the options — never your
own pick. A cascaded seat enters its members too. Member code is
edited only in the member worktree paths `status` prints, never in
a member's main checkout. A standing member worktree is switched
into like the home one; an absent one re-attaches with
`archi worktree mint <slug> --repos a,b` — it extends the seat,
never recreates it, and a refused baseline routes through
`--base <member>=<branch>` as `archi.md` teaches. `archi repo ls`
is the health read on the way in: reachability, cleanliness,
baselines. A member checkout outside the session's working
directories is added to them before any git runs there — `/add-dir`
in Claude Code, the equivalent elsewhere. Git refused there is the
session's boundary, not the tool's, and handing git back to the
operator is not the repair. Then the chosen workflow skill runs its
own opening check.
