---
node: Scaffold
owns: [the-briefing-says-what-help-does-not, a-skill-migrates-a-standing-project-into-the-wing]
---

# t5 — Scaffold

the briefing shrinks and the migration skill learns to ask

## Spec

- `Scaffold`
- `Function type_of Scaffold`
- `Cli.drive consult(->Command, <-Report) Scaffold.stand_up`

## Inputs


## Outputs

- crates/archi/src/scaffold.rs
- crates/archi/tests/init_e2e.rs
- skills/archi-migrate-world.md

## Stack

- the block text is the `claude_block` constant; the skill texts are the embedded markdown under `skills/`
- the length assertion is a line count over the installed block, so the rule cannot rot silently

## Verifications

### the-briefing-says-what-help-does-not

- test — the installed block lists no command flags
- test — the installed block names no skill by file path
- test — the block states the no-model-nouns rule
- test — the block states that spec work returns as files
- test — the block is shorter than twenty lines

### a-skill-migrates-a-standing-project-into-the-wing

- test — the skill text tells the reader to offer options rather than ask open questions
- test — the skill text tells the reader to ask again after a first "nothing would falsify it"
- test — the skill text names the test suites as a third place candidates come from
- test — the skill text still names the workaround as the gate
- test — `init` and `sync-skills` install it byte-equal to the embedded copy
