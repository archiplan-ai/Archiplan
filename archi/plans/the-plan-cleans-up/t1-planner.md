---
node: Planner
owns: [the-plan-cleans-up-after-itself]
facts: [an-assistant-handed-the-whole-of-a-job-does-part-of-it@549190]
---

# t1 — Planner

the close deletes what it consumed

## Spec

- `Planner`
- `Service type_of Planner`
- `Cli.drive consult(->Command, <-Report) Planner.advance`
- `Links.Capture`

## Inputs

## Outputs

- crates/archi/src/plans/mod.rs
- crates/archi/src/links/capture.rs
- crates/archi/tests/plan_e2e.rs

## Stack

- the record answers where (`link ls --spec` per ref): the close is
  `crates/archi/src/plans/mod.rs#next`, the CLI arm is `main.rs#run_plan` (not an output —
  nothing there changes)
- the deletion happens inside a successful close only, after capture consumed the files: a
  blocked close (declaration gate, coverage gate, drift gate) deletes nothing, because the
  retry reads the same files
- the paths live in `crates/archi/src/links/capture.rs` — `index_path` and `declares_rel`
  are private; expose a `remove_wave_files(root, plan, wave, tasks)` there rather than
  re-deriving paths in `plans/mod.rs`, so writer and deleter share one path source
- completion (the step that prints DONE) removes the now-empty `waves/` dir; `plan reset`
  at `plans/mod.rs:1978` already does `remove_dir_all` and stays as is
- a wave closed by an older binary left files behind; a later completion of that plan
  removes `waves/` whole, so the old leftovers go with it

## Verifications

### the-plan-cleans-up-after-itself

- test — a successful wave close deletes that wave's index and declaration files, and the
  next wave's files stand untouched
- test — a blocked close deletes nothing, and the retry closes on the same files
- test — at DONE the plan folder holds no `waves/` at all
- test — `plan reset` still clears `waves/` whole
