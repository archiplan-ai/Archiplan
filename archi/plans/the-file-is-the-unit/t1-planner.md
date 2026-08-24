---
node: Planner
owns: [the-file-in-the-delta-is-the-unit-the-gate-demands]
facts: [an-assistant-handed-the-whole-of-a-job-does-part-of-it@549190]
---

# t1 — Planner

the gate demands the delta's files against the declarations

## Spec

- `Planner`
- `Service type_of Planner`
- `Cli.drive consult(->Command, <-Report) Planner.advance`
- `Cli.drive consult(->Command, <-Report) Planner.author`
- `Links.Capture`
- `Planner.run_capture consult(->ItemHashIndex, <-LinkEvent) Links.capture`

## Inputs

## Outputs

- crates/archi/src/links/capture.rs
- crates/archi/src/plans/mod.rs
- crates/archi/tests/plan_e2e.rs

## Stack

- the term test is `ref_terms`, `item_terms`, `terms_into`, `keep` and `STOPPED` in
  `crates/archi/src/links/capture.rs` — all of it goes
- the loop that reads `## Outputs` is at `crates/archi/src/links/capture.rs:956`; it computes
  claimants, leftovers, pressed and the shared-file note. The claim map and `pressed` go with
  it; what replaces the loop is the plain list of files the delta touched
- `CaptureOutcome` loses `pressed` and `suppressed`, and `Suppressed` retires; the render at
  `crates/archi/src/links/capture.rs:1096` loses the no-signal count
- the gate is `gate_coverage` in `crates/archi/src/plans/mod.rs`; it now takes the delta's
  files and the union of the declarations, and refuses on a file no entry names
- the declaration reader already parses `symbol` as `<file>#<symbol>` or a bare file, so the
  file side of an entry is `Anchor::parse(symbol).file`

## Verifications

### the-file-in-the-delta-is-the-unit-the-gate-demands

- test — a wave whose delta holds a file no declaration names refuses, and the refusal names
  that file
- test — the refusal names the file and no task, with two tasks in flight
- test — a declaration naming a file outside its task's `## Outputs` satisfies the gate for
  that file
- test — one file named by two entries against two different elements satisfies the gate once
- test — a wave whose declarations name every file in the delta closes with no spec ref
  demanded
- test — `plan next` prints no `no-signal pair` count
