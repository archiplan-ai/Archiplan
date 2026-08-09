---
kind: functional
origin: intent
satisfied-by: [Cli, Seats.Registry]
deferred:
---

# The registry moves by verbs

The registry is operated only through the CLI, never by hand: a verb lists every row with its
binding and state, a verb closes one; mint writes rows (worktrees-mint-on-demand), the landing
closes them once integration proves the work in place (merge-retires-the-worktree). No verb
deletes a row (nothing-leaves-the-registry).

## System Context

Same ground rule as every lifecycle store: files are the truth, verbs are the only writers.
The listing is the operator's view over the work this machine carries and carried
(the-listing-carries-the-status); the close verb is the manual exit for work abandoned instead
of landed, and for what self-healing against `git worktree list` cannot decide
(the-registry-binds-the-worktree).

## Satisfy

`Cli.worktree` is the only writer's surface: `ls` lists every row with its binding and state,
`close` ends one, mint writes and re-opens, the landing closes — no hand edits, no deletions.

- test — mint records, ls shows, close marks the row and removes the worktree (`mint_without_a_plan_binds_spec_work_and_close_retires_it`)
- test — close cascades over member worktrees (`close_cascades_over_member_worktrees`)
