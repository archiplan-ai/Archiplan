---
node: Planner
owns: [the-gate-refusal-names-the-repair-that-stands]
facts: [an-assistant-handed-the-whole-of-a-job-does-part-of-it@549190]
---

# t1 — Planner

the coverage refusal names the repair that exists

## Spec

- `Planner`
- `Service type_of Planner`
- `Cli.drive consult(->Command, <-Report) Planner.advance`
- `Cli.drive consult(->Command, <-Report) Planner.author`

## Inputs

## Outputs

- crates/archi/src/plans/mod.rs
- crates/archi/tests/plan_e2e.rs

## Stack

- the refusal is built at `crates/archi/src/plans/mod.rs:1702`, in the arm that runs when
  `gaps` is not empty; the sentence to replace is "review the captured candidates (`archi
  link ls --evidence`), assert the load-bearing ones (`archi link confirm <id>`), then
  re-run `archi plan next`"
- the repair that stands is `archi link add <ref> <file#symbol> --kind indirect`; the same
  function already prints that form for the refs the delta does not press, a few lines
  below, so the two halves of the message should read as one
- the doc comment on `pub fn next` at `crates/archi/src/plans/mod.rs:1777` documents the
  same retired step: "re-runnable: review (`link confirm`), then run it again"
- the test home is `crates/archi/tests/plan_e2e.rs`; the existing
  `the_plan_loop_produces_the_links_its_gate_demands` already drives `plan next` to this
  refusal and asserts on its text at the line holding "coverage of the refs this delta
  presses is incomplete"

## Verifications

### the-gate-refusal-names-the-repair-that-stands

- test — the coverage refusal of `plan next` names `archi link add` as the repair, with the
  ref and the anchor it takes
- test — the coverage refusal names neither `link ls --evidence` nor `link confirm`
