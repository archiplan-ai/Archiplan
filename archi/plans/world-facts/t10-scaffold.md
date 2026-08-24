---
node: Scaffold
owns: [the-briefing-puts-the-world-in-the-loop, a-skill-migrates-a-standing-project-into-the-world]
---

# t10 — Scaffold

the briefing and the migration skill

## Spec

- `Scaffold`
- `Function type_of Scaffold`
- `Cli.drive consult(->Command, <-Report) Scaffold.stand_up`

## Inputs

- from t5 — the verb surface and its flags, which the briefing must describe accurately

## Outputs

- crates/archi/src/scaffold.rs
- crates/archi/tests/init_e2e.rs

## Stack

- skill texts are embedded in the binary and installed byte-equal, as the other skills are
- the archi workflow skill gains a capture step before requirements are derived
- `archi-migrate-world` is a new embedded skill file installed beside the others

## Verifications

### the-briefing-puts-the-world-in-the-loop

- test — init_e2e: the installed briefing names the `world` verb and its subcommands
- test — init_e2e: the briefing states the no-model-nouns rule
- test — init_e2e: the briefing places the capture step before requirements are derived
- test — init_e2e: `sync-skills` on a pre-world project reports the briefing updated

### a-skill-migrates-a-standing-project-into-the-world

- test — init_e2e: `init` and `sync-skills` install the migration skill byte-equal to the embedded copy
- test — init_e2e: the skill text names the workaround as the gate that stops a fact being written
- test — init_e2e: the skill text requires a brief of what did not map
- test — world_e2e: a fact minted through the skill names its origin file in `sources` and reports nothing
