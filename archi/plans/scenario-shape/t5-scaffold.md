---
node: Scaffold
owns: [the-briefing-says-what-help-does-not]
---

# t5 — Scaffold

the briefing and the skills describe the shape that ships

## Spec

- `Scaffold`
- `Function type_of Scaffold`
- `Cli.drive consult(->Command, <-Report) Scaffold.stand_up`

## Inputs

- from t1 — the shape the texts must describe

## Outputs

- crates/archi/src/scaffold.rs
- skills/archi.md
- skills/archi-migrate-world.md
- crates/archi/tests/init_e2e.rs

## Stack

- the archi skill's world step and the migration skill both spell the scenario shape out
- the briefing block keeps its length rule; the shape is one line inside it or none at all

## Verifications

### the-briefing-says-what-help-does-not

- test — init_e2e: the installed archi skill describes a scenario as a heading and its steps
- test — init_e2e: neither installed skill mentions `Feature:` or `Scenario:` as something to write
- test — init_e2e: the migration skill's example fact uses the new shape
- test — init_e2e: the block is still shorter than twenty lines
- test — init_e2e: every installed text is byte-equal to its embedded copy
