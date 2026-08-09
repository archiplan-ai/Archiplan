---
node: Cli
owns: [the-listing-carries-the-status, the-registry-moves-by-verbs]
---

# t2 — Cli

`worktree drop` becomes `worktree close`: same cascade, same refusal on
an unclean tree, but it closes the row instead of deleting it. A bare
`drop` refuses toward `close` so muscle memory is not silence, and
USAGE names only `close`. `ls` renders the state of every row and of
every member under it — live, waiting on a named branch, or closed —
and takes `--status active|closed|all` (default all, an unknown value
refuses naming the three), composing with the existing filters. The
sweep runs on the registry-reading commands and prints one line for
what it freed. `status` in an unbound checkout names the rows that
stand, so a checkout carrying nothing still points at the work.

## Spec

- `Cli`
- `Service type_of Cli`
- `Agent.drive invoke(->Command, <-Report) Cli.build`
- `Agent.drive invoke(->Command, <-Report) Cli.check`
- `Agent.drive invoke(->Command, <-Report) Cli.incidence`
- `Agent.drive invoke(->Command, <-Report) Cli.init`
- `Agent.drive invoke(->Command, <-Report) Cli.link`
- `Agent.drive invoke(->Command, <-Report) Cli.nkp`
- `Agent.drive invoke(->Command, <-Report) Cli.plan`
- `Agent.drive invoke(->Command, <-Report) Cli.query`
- `Agent.drive invoke(->Command, <-Report) Cli.read`
- `Agent.drive invoke(->Command, <-Report) Cli.repo`
- `Agent.drive invoke(->Command, <-Report) Cli.req`
- `Agent.drive invoke(->Command, <-Report) Cli.search`
- `Agent.drive invoke(->Command, <-Report) Cli.session`
- `Agent.drive invoke(->Command, <-Report) Cli.status`
- `Agent.drive invoke(->Command, <-Report) Cli.stress`
- `Agent.drive invoke(->Command, <-Report) Cli.update`
- `Agent.drive invoke(->Command, <-Report) Cli.version`
- `Agent.drive invoke(->Command, <-Report) Cli.worktree`

## Inputs

- from t1 — the row state, the landing record and the integration probe the surface renders

## Outputs

- crates/archi/src/main.rs
- crates/archi/tests/worktree_e2e.rs

## Stack

- run_worktree's ls/drop arms and the USAGE block in main.rs
- run_status's unbound branch — today it says only "unbound"

## Verifications

### the-listing-carries-the-status

- test — worktree_e2e: ls shows a waiting row with the branch it compared against, a live row and a closed row; `--status` keeps each in turn and an unknown value refuses naming active, closed and all; status in an unbound checkout names the standing rows

### the-registry-moves-by-verbs

- test — worktree_e2e: mint records, ls shows, close marks the row and removes the worktree (`mint_without_a_plan_binds_spec_work_and_close_retires_it`); close cascades over member worktrees (`close_cascades_over_member_worktrees`); a bare `drop` refuses toward `close`
