---
node: Planner
owns: [a-task-carries-the-facts-that-cover-its-node, the-plan-closes-on-the-world-s-scenarios, the-close-re-reads-the-world-and-says-what-moved, the-block-marks-what-lies-outside-the-plan, the-close-gates-on-anchored-scenarios, a-plan-s-own-scenarios-block-retires]
---

# t8 — Planner

the plan carries and closes on covering facts

## Spec

- `Planner`
- `Service type_of Planner`
- `Cli.drive consult(->Command, <-Report) Planner.advance`
- `Cli.drive consult(->Command, <-Report) Planner.author`

## Inputs

- from t3 — the loaded facts keyed by what they cover
- from t7 — scenario spec refs, for the anchored-scenario gate

## Outputs

- crates/archi/src/plans/mod.rs
- crates/archi/src/plans/records.rs
- crates/archi/tests/plan_e2e.rs

## Stack

- covering facts resolve per task node at author time and re-resolve on verify and repin
- the closing block is collected at close, never copied into the record
- the anchored gate reads the folded link set the coverage gate already consults

## Verifications

### a-task-carries-the-facts-that-cover-its-node

- test — plan_e2e: a task over a covered node lists that fact on `plan task show`
- test — plan_e2e: `plan verify` flags a task whose named fact was retired since the pin
- test — plan_e2e: `plan repin` re-resolves the covering facts against the new version

### the-plan-closes-on-the-world-s-scenarios

- test — plan_e2e: a plan touching a covered node prints that fact's scenarios at close
- test — plan_e2e: a fact covering two of the plan's nodes prints once, not twice
- test — plan_e2e: `plan scenarios list` and the close step read the same set

### the-close-re-reads-the-world-and-says-what-moved

- test — plan_e2e: a fact retired since authoring is named as drift at close
- test — plan_e2e: a fact whose scenarios changed since authoring is named as drift
- test — plan_e2e: the plan record holds fact slugs and no scenario text

### the-block-marks-what-lies-outside-the-plan

- test — plan_e2e: a fact covering four nodes on a one-node plan prints the other three as outside
- test — plan_e2e: a fact whose covered nodes the plan all holds prints with no mark

### the-close-gates-on-anchored-scenarios

- test — plan_e2e: a block whose scenarios all carry links latches closed
- test — plan_e2e: one unanchored scenario refuses the latch and is named
- test — plan_e2e: `plan reset` clears the latch after a refusal

### a-plan-s-own-scenarios-block-retires

- test — plan_e2e: a pre-world plan closes with no block and raises no finding
- test — plan_e2e: a post-world plan with an empty collected block refuses the final latch
- test — plan_e2e: no verb writes or deletes `scenarios.md`
