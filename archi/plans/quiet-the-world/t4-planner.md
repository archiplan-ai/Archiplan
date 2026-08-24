---
node: Planner
owns: [a-plan-s-own-scenarios-block-retires]
---

# t4 — Planner

the empty-block refusal needs a world to refuse against

## Spec

- `Planner`
- `Service type_of Planner`
- `Cli.drive consult(->Command, <-Report) Planner.advance`
- `Cli.drive consult(->Command, <-Report) Planner.author`

## Inputs

- from t1 — the scenario digest function, so the plan's drift and the link's grade agree

## Outputs

- crates/archi/src/plans/mod.rs
- crates/archi/tests/plan_e2e.rs

## Stack

- the refusal asks whether the tree holds a world at all before it fires
- the drift fingerprint calls t1's function instead of the private one written here

## Verifications

### a-plan-s-own-scenarios-block-retires

- test — plan_e2e: a post-world plan on a tree with no world facts at all closes without a refusal
- test — plan_e2e: a post-world plan on a tree that has a world refuses until a fact covers one of its nodes
- test — plan_e2e: a pre-world plan closes with no block and raises no finding
- test — plan_e2e: no verb writes or deletes `scenarios.md`
- test — the drift fingerprint and the link digest agree on the same block
