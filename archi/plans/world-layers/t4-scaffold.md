---
node: Scaffold
owns: [the-briefing-says-what-help-does-not]
---

# t4 — Scaffold

the skills describe the four layers

## Spec

- `Scaffold`
- `Function type_of Scaffold`
- `Cli.drive consult(->Command, <-Report) Scaffold.stand_up`

## Inputs

- from t1 — the four layers and the source rule the texts must describe

## Outputs

- crates/archi/src/scaffold.rs
- skills/archi.md
- skills/archi-migrate-world.md
- crates/archi/tests/init_e2e.rs

## Stack

- the world step of the workflow skill names the four folders and what each holds
- the migration skill stops writing an intent into `sources` and says an unobserved claim carries none

## Verifications

### the-briefing-says-what-help-does-not

- test — init_e2e: the installed workflow skill names the four folders and what each holds
- test — init_e2e: neither installed skill tells the reader to put a path outside the world into `sources`
- test — init_e2e: the migration skill says a claim lifted from prose carries no source
- test — init_e2e: the block is still shorter than twenty lines
- test — init_e2e: every installed text is byte-equal to its embedded copy
