---
node: Seats
owns: [the-refresh-never-blocks-the-mint, the-branch-point-follows-the-divergence, a-seat-sits-beside-the-main-checkout]
---

# t1 — Seats

Before it creates anything, the mint refreshes the base it will branch
from — the home branch, and every cascaded member's base — with a fetch
of that branch alone. Best-effort: no remote, no upstream, no network or
a refusal degrades to the local ref, and `--no-fetch` skips the attempt.
Nothing is ever pulled, so no working tree moves.

Then the branch point follows the divergence against the remote
counterpart: behind takes the remote ref, ahead or diverged takes the
local branch, no upstream or no difference takes the local ref. An
explicit `--base` overrides the decision entirely. The mint report gains
the branch point and the reason: the ref it grew from, its short commit,
and the divergence that decided it.

The home folder anchors on the repository's main checkout, the same
anchor the cascade already uses for members, so a mint run from inside a
seat lands its folder beside the main checkout instead of nesting.

## Spec

- `Seats`
- `Seats.Mint`
- `Service type_of Seats`
- `Storage type_of Seats.Registry`
- `Cli.drive consult(->Command, <-Report) Seats.bind`
- `Cli.drive consult(->Command, <-Report) Seats.guard`
- `Cli.drive consult(->Command, <-Report) Seats.land`
- `Cli.drive consult(->Command, <-Report) Seats.mint`
- `Cli.drive consult(->Command, <-Report) Seats.survey`
- `Cli.drive consult(->Command, <-Report) Seats.verdict`

## Inputs

## Outputs

- crates/archi/src/worktrees.rs
- crates/archi/src/main.rs
- crates/archi/tests/worktree_e2e.rs

## Stack

- gitcmd's run/out plumbing for `fetch`, `rev-list --left-right --count`, `rev-parse`
- the main-checkout anchor already used by the member cascade
- --no-fetch in the mint arm of run_worktree

## Verifications

### the-refresh-never-blocks-the-mint

- test — worktree_e2e: a mint against an unreachable remote succeeds, branches from the local ref and names it; `--no-fetch` skips the attempt; a member whose checkout is dirty or stands on another branch is refreshed with its working tree untouched

### the-branch-point-follows-the-divergence

- test — worktree_e2e: a base behind its remote branches from the remote ref; a base ahead branches locally and the report names the count; a diverged base branches locally and names both counts; `--base` overrides all of it

### a-seat-sits-beside-the-main-checkout

- test — worktree_e2e: a mint run from inside a seat lands its folder beside the main checkout and the registry records that path
