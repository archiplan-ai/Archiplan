---
node: Planner
owns: [the-closing-step-hands-back-the-work]
---

# t3 — Planner

the closing step prints the Gherkin, the state and the command

## Spec

- `Planner`
- `Service type_of Planner`
- `Cli.drive consult(->Command, <-Report) Planner.advance`
- `Cli.drive consult(->Command, <-Report) Planner.author`

## Inputs

- from t1 — the per-scenario grade, so the state printed beside a scenario is the corrected one

## Outputs

- crates/archi/src/plans/mod.rs
- crates/archi/tests/plan_e2e.rs

## Stack

- the closing render reads the folded link set the anchored gate already consults
- the ref string is quoted for a shell, because a scenario name carries spaces
- the same three states print from `plan verify` before the last wave closes

## Verifications

### the-closing-step-hands-back-the-work

- test — plan_e2e: the closing step prints the feature line and every step of each collected scenario
- test — plan_e2e: an unanchored scenario prints a `link add` line whose ref is single-quoted
- test — plan_e2e: that printed line is executed through `sh` and the next `plan next` no longer names that scenario
- test — plan_e2e: an anchored clean scenario prints its state and carries no command
- test — plan_e2e: an anchored scenario whose step was reworded prints as drifted, naming the scenario side
- test — plan_e2e: `plan verify` prints the same three states while a wave is still open
