---
node: Seats
owns: [a-landing-carries-fresh-baselines]
---

# t1 — Seats

The gate and the note. In `worktrees.rs::merge`: after the plan-close gate,
for every member of the binding compare the member worktree HEAD with the
baseline the seat's latest version records; any mismatch (or missing
baseline) joins one batched refusal naming each member and the repair —
`archi version anchor --repo <member>` in the seat — before any push or
merge, on the `--to` path alike. In `plan_cascade`'s auto-base arm: when the
baseline is an ancestor but behind the branch tip, print a note with the
commit count — proceed, never refuse. Tests in worktree_e2e: un-anchored
member work refuses the merge and lands after anchoring; `--to` runs the
same gate; the stale-but-reachable mint prints behind-by-N.

## Spec

- `Seats`
- `Seats.Landing`
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

## Verifications

### a-landing-carries-fresh-baselines

- cargo test worktree_e2e — a merge with un-anchored member work refuses naming the member and the anchor recipe; after anchor it lands
- cargo test worktree_e2e — the --to landing runs the same gate
- cargo test worktree_e2e — a mint from a behind-but-reachable baseline proceeds with the behind-by-N note
