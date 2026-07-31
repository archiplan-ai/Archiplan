---
node: Seats
owns: [an-off-baseline-base-says-so]
---

# t1 — Seats

In `plan_cascade`'s explicit `--base` arm: read the member's recorded
baseline from the pinned version (the same source the auto arm uses).
When one is recorded, ask git whether the named base branch contains it
— `merge-base --is-ancestor <baseline> <branch>`. On "no", and when the
object is missing entirely, print one stdout note and continue:
`note: member <name>: the named base \`<branch>\` does not contain the
recorded baseline <sha7> — continuing off the audited line`. No
baseline recorded — silent, as today. Never a refusal on this arm.

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

- git merge-base --is-ancestor via the module's Command plumbing
- the pinned version's Entry.commits — the auto arm's baseline source, reused

## Verifications

### an-off-baseline-base-says-so

- test — worktree_e2e: `--base <member>=<branch>` onto a branch without the recorded baseline prints the note with member, branch and sha7; onto the branch carrying the baseline stays silent; with no baseline recorded stays silent
