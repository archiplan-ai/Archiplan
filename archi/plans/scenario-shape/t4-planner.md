---
node: Planner
owns: [the-closing-step-hands-back-the-work]
---

# t4 — Planner

an anchored scenario names its file and asks for the re-read

## Spec

- `Planner`
- `Service type_of Planner`
- `Cli.drive consult(->Command, <-Report) Planner.advance`
- `Cli.drive consult(->Command, <-Report) Planner.author`

## Inputs

- from t3 — the per-scenario grade the state is read from

## Outputs

- crates/archi/src/plans/mod.rs
- crates/archi/tests/plan_e2e.rs
- crates/archi/tests/check_e2e.rs
- crates/archi/tests/world_e2e.rs
- crates/archi/tests/read_e2e.rs

## Stack

- an anchored scenario prints its `file#symbol` and the sentence asking for the re-read
- a drifted one keeps naming the side that moved, beside the anchor
- every suite still carrying an old-shape fixture moves with it: the shape changed under them and nothing else did

## Verifications

### the-closing-step-hands-back-the-work

- test — plan_e2e: an anchored scenario prints the file and symbol it anchors and asks for the re-read
- test — plan_e2e: an anchored scenario whose step was reworded prints the anchor and the side that moved
- test — plan_e2e: an unanchored scenario is named new and prints the quoted `link add`
- test — plan_e2e: the printed command runs through a real `sh` and the next `plan next` stops naming it
- test — plan_e2e: `plan verify` answers the same way while a wave is open
- test — the whole suite is green: every fixture writing a scenario writes the new shape
