---
node: Scaffold
owns: [the-briefing-says-what-help-does-not]
---

# t2 — Scaffold

the skills ask for the workaround and stop asking what would end it

## Spec

- `Scaffold`
- `Function type_of Scaffold`
- `Cli.drive consult(->Command, <-Report) Scaffold.stand_up`

## Inputs


## Outputs

- crates/archi/src/scaffold.rs
- skills/archi.md
- skills/archi-migrate-world.md
- crates/archi/tests/init_e2e.rs

## Stack

- the world step of the workflow skill and the interview of the migration skill both spell the sections out
- the migration skill's interview loses the killer question and keeps the workaround as its gate
- both texts gain the rule that a fact names no person and quotes nobody

## Verifications

### the-briefing-says-what-help-does-not

- test — init_e2e: the installed workflow skill names `What people do instead` as a required section
- test — init_e2e: neither installed skill asks the reader what would end the fact
- test — init_e2e: both installed skills state that a fact names no person and quotes nobody
- test — init_e2e: the migration skill's interview keeps the workaround as the gate that stops a fact being written
- test — init_e2e: the block is still shorter than twenty lines
- test — init_e2e: every installed text is byte-equal to its embedded copy
