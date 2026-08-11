---
node: Gherkin
owns: [the-grammar-is-a-named-subset, scenarios-parse-or-the-check-fails]
---

# t2 — Gherkin

the scenario grammar: parse, then hold the named subset

## Spec

- `Gherkin`
- `Function type_of Gherkin`
- `DocsCompiler.parse_scenarios consult(->WorldDoc, <-Diagnostics) Gherkin.parse`

## Inputs

- from t1 — the `Scenarios` block text and its offset inside the fact file

## Outputs

- crates/archi/src/docs/gherkin.rs
- crates/archi/Cargo.toml

## Stack

- the gherkin crate parses the block; its AST carries 1-indexed line and column
- a subset pass walks that AST and rejects every construct outside the six keywords and tags
- block-offset arithmetic maps a crate location onto the line inside the fact file

## Verifications

### the-grammar-is-a-named-subset

- test — a `Feature` with two `Scenario`s and their Given/When/Then/And steps parses
- test — `Background`, `Rule`, `Scenario Outline`, `Examples`, a docstring and a data table each raise a located error naming the construct
- test — the error text lists the six keywords the subset holds
- test — `But` and `*` raise the same error as any other unknown step keyword

### scenarios-parse-or-the-check-fails

- test — a malformed step raises E_DOC and check exits non-zero
- test — the reported line is the line in the fact file, not the line inside the block
- test — a `Scenarios` heading holding prose instead of Gherkin raises E_DOC
- test — an empty `Scenarios` block raises E_DOC

