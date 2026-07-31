---
node: Seats
owns: [the-cascade-refuses-a-side-worktree-base]
---

# t2 — Seats

In `plan_cascade`'s member resolution: when a member without an
explicit `--base` and not already in the seat's binding resolves to a
linked worktree, refuse — batched with the other member refusals, in
the standing style. The message calls the overlay row stale, names the
branch checked out there, and gives both repairs verbatim: `archi repo
map <member> <main-checkout-path>` and `--base <member>=<branch>`. An
explicit `--base` for that member skips the gate entirely. Placement:
the new member worktree's sibling folder derives from the repo's main
checkout toplevel, never from the mapped path, on every arm.

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
- crates/archi/tests/worktree_e2e.rs

## Stack

- git rev-parse --git-dir / --git-common-dir — the same detection as t1, local helper in worktrees.rs
- git worktree list --porcelain — main checkout for the refusal text and the placement anchor
- batched member refusals — the plan_cascade preflight style already in place

## Verifications

### the-cascade-refuses-a-side-worktree-base

- test — worktree_e2e: a member mapped to a linked worktree refuses the mint; the message names `repo map` with the main checkout and `--base <member>=<branch>`
- test — worktree_e2e: the same mint with `--base <member>=<branch>` proceeds; the member worktree lands in the main checkout's sibling folder, not beside the mapped path
- test — worktree_e2e: a re-mint extending a seat whose member already rides its own worktree stays silent — the seat's worktrees are never gated
