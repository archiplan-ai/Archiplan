---
node: Planner
owns: [a-cleanup-wave-precedes-the-scenarios]
---

# t1 — Planner

The plan lifecycle gains one stage between the last wave and the
scenarios, mirroring the scenarios latch exactly. `plan next` after the
last wave closes prints the cleanup block once — the sweep's mandate in
two or three lines: one sub-agent, the unit's whole delta, fold the
twins, zero behavior change, tests green with no assertion edits, an
empty sweep is one line. The next `plan next` records the latch and
prints the scenarios block as today. State rides a `cleanup_closed`
bool in `state.json` with a serde default, so legacy files read; the
records parser accepts it. No capture machinery for the stage — task
`outputs` already claim the touched files for the audit.

## Spec

- `Planner`
- `Service type_of Planner`
- `Cli.drive consult(->Command, <-Report) Planner.advance`
- `Cli.drive consult(->Command, <-Report) Planner.author`

## Inputs

## Outputs

- crates/archi/src/plans/mod.rs
- crates/archi/src/plans/records.rs
- crates/archi/src/main.rs
- crates/archi/tests/plan_e2e.rs

## Stack

- the scenarios_displayed/scenarios_closed latch pattern in plans/mod.rs — mirror it
- serde default on the new state field — deny_unknown_fields stays

## Verifications

### a-cleanup-wave-precedes-the-scenarios

- test — plan_e2e: after the last wave `plan next` prints the cleanup block once, the next call prints the scenarios, the one after completes; a legacy state.json without the field still reads and completes
