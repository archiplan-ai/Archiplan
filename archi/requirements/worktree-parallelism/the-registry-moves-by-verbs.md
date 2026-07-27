---
kind: functional
origin: intent
satisfied-by: [Cli, Seats.Registry]
deferred:
---

# The registry moves by verbs

The registry is operated only through the CLI, never by hand: a verb lists every worktree with
its binding, a verb drops a stale entry; mint writes entries (worktrees-mint-on-demand), merge
clears them (merge-retires-the-worktree).

## System Context

Same ground rule as every lifecycle store: files are the truth, verbs are the only writers.
The listing is the operator's view over parallel work in flight; the drop verb is the manual
repair for what self-healing against `git worktree list` cannot decide
(the-registry-binds-the-worktree).

## Satisfy

`Cli.worktree` is the only writer's surface: `ls` lists every worktree with its binding,
`drop` repairs a stale row, mint writes, merge clears — no hand edits.

- test — mint records, ls shows, drop retires the row and the worktree (`mint_without_a_plan_seats_spec_work_and_drop_retires_it`)
- test — drop cascades over member worktrees (`drop_cascades_over_member_worktrees`)
