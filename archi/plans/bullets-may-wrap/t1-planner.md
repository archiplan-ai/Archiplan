---
node: Planner
owns: [a-record-bullet-may-wrap, the-empty-block-asks-only-where-a-condition-is-owed]
facts: [an-assistant-handed-the-whole-of-a-job-does-part-of-it@549190]
---

# t1 — Planner

a bullet may wrap, and an empty block asks only where a condition is owed

## Spec

- `Planner`
- `Service type_of Planner`
- `Cli.drive consult(->Command, <-Report) Planner.advance`
- `Cli.drive consult(->Command, <-Report) Planner.author`
- `PlanFile`
- `Data type_of PlanFile`

## Inputs

## Outputs

- crates/archi/src/plans/records.rs
- crates/archi/src/plans/mod.rs
- crates/archi/tests/plan_e2e.rs
- skills/archi-plan.md

## Stack

- `bullet()` at `crates/archi/src/plans/records.rs:256` strips `- ` from one line and refuses anything else; the line loops that call it sit around lines 287, 371, 445 and 650, and each mixes bullet sections with prose sections
- the fold applies to bullet sections alone — a bare line in a prose section is prose and stays one line, exactly as today
- a section's bullets are its lines split at every line opening with `- `; each piece collapses its line breaks and blank lines to single spaces
- the line number carried into `shape_err` stays the line the bullet opened on, so a refusal never points at a continuation
- `EMPTY_BLOCK` at `crates/archi/src/plans/mod.rs:1031` fires on emptiness alone; the owing set is the task nodes minus the `Data`-classified and minus the `.worldignore` entries, which is the same exclusion `world_check::unreached` already computes for `version save`
- `skills/archi-plan.md` carries the rule the first requirement retires — "A record bullet is one bullet on one line. The bullets do not wrap" — and it goes with the fix; the skills compile into the binary, so `cargo build --release` then `archi sync-skills`

## Verifications

### a-record-bullet-may-wrap

- test — a stack bullet wrapped onto a second line parses as one bullet, joined by one space
- test — the same for spec, inputs, outputs, architecture and verification bullets
- test — a bullet wrapped onto three lines, and one wrapped with a blank line inside it, both parse as one bullet
- test — blank lines between bullets change nothing
- test — prose sections keep every line, wrapped or not, exactly as today
- test — a malformed bullet is refused at the line it opened on, not at a continuation
- test — every plan record standing in `archi/plans/` parses byte-identically to today, proven against the real tree
- test — the planning skill no longer carries the one-line rule

### the-empty-block-asks-only-where-a-condition-is-owed

- test — a plan whose every task node is `Data` closes on an empty block with no refusal
- test — the same for task nodes named in `.worldignore`
- test — a plan with one behaviour node no fact covers refuses, and the refusal names that node
- test — the refusal names the owing nodes, not the plan
- test — a non-empty block closes as it does today
- test — a tree with no world facts at all closes as it does today
