---
node: Cli
owns: [one-verb-lists-the-decisions-on-an-element]
---

# t1 — Cli

decision ls: the read over the links field

## Spec

- `Cli`
- `Service type_of Cli`
- `Agent.drive invoke(->Command, <-Report) Cli.query`
- `DocsCompiler`

## Inputs

## Outputs

- crates/archi/src/main.rs
- crates/archi/src/docs/mod.rs
- crates/archi/src/tradeoffs.rs
- crates/archi/tests/mint_e2e.rs

## Stack

- the record answers where: the decision files are read today by
  `crates/archi/src/tradeoffs.rs#revealed` (loads prefer/over per file) and indexed by
  search; the listing's serving side goes beside one of them — mirror how `req ls` serves
  from `docs/mod.rs#serve_requirements`
- the verb arm goes in `crates/archi/src/main.rs` beside the other listings; there is no
  `decision` arm today, so the usage block and module doc gain the verb whole:
  `archi decision ls [--links <name>] [--json]`
- `--links <name>`: an element resolves against the live model; a non-element resolves
  against the doc slugs (requirements, stressors, world facts, decisions); neither —
  refusal naming it, in the shape `req ls --satisfies` refuses
- the row: slug, `prefer -> over` (empty sides shown empty), first phrase of the rationale
  — one line, mirror `req ls`'s render and its `--json` envelope
- tests in `crates/archi/tests/mint_e2e.rs` beside the `req ls` family, on its
  `listing_project` fixture extended with two decision files

## Verifications

### one-verb-lists-the-decisions-on-an-element

- test — `decision ls` prints one row per decision file, and the count matches the files
  on disk
- test — `--links <element>` narrows to the decisions naming it, and `--links <slug>`
  does the same for a doc slug
- test — `--links` with a name that resolves as neither element nor record refuses,
  naming it
- test — `--json` carries the same rows as the render
