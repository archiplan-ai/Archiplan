---
node: Scaffold
owns: [the-plan-collects-its-scenarios-and-authors-none]
---

# t2 — Scaffold

the planning skill stops asking for a block no verb reads

## Spec

- `Scaffold`
- `Function type_of Scaffold`
- `Cli.drive consult(->Command, <-Report) Scaffold.stand_up`

## Inputs


## Outputs

- crates/archi/src/scaffold.rs
- skills/archi-plan.md
- crates/archi/tests/init_e2e.rs

## Stack

- the planning skill is an embedded markdown text installed byte-equal, as the others are
- the section to rewrite is the one that today tells the author to walk the architecture and write one bullet per flow

## Verifications

### the-plan-collects-its-scenarios-and-authors-none

- test — init_e2e: the installed planning skill holds no instruction to write `scenarios.md`
- test — init_e2e: it names the world facts covering the plan's task nodes as the source of the closing block
- test — init_e2e: it says an empty block is spec work, not a blank to fill
- test — init_e2e: `sync-skills` on a project installed before this change reports the planning skill updated
- test — init_e2e: the installed text is byte-equal to the embedded copy
