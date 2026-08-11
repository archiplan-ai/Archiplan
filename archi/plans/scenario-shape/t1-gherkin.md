---
node: Gherkin
owns: [the-grammar-is-a-named-subset, scenarios-parse-or-the-check-fails]
---

# t1 — Gherkin

a heading names the scenario, four keywords open its steps

## Spec

- `Gherkin`
- `Function type_of Gherkin`
- `DocsCompiler.parse_scenarios consult(->WorldDoc, <-Diagnostics) Gherkin.parse`

## Inputs


## Outputs

- crates/archi/src/docs/gherkin.rs
- crates/archi/Cargo.toml

## Stack

- a level-three heading opens a scenario; `Given`, `When`, `Then`, `And` open its step lines
- `Feature:` and `Scenario:` get their own error naming where each belongs — the fact's title, the heading
- the `gherkin` crate leaves `Cargo.toml`; the reader is a line walk over the block

## Verifications

### the-grammar-is-a-named-subset

- test — unit test in `gherkin.rs`: two headings and their steps parse into two named scenarios with their file lines
- test — unit test: a `Feature:` line raises `E_DOC` whose text names the fact's title as its place
- test — unit test: a `Scenario:` line raises `E_DOC` whose text names the heading as its place
- test — unit test: a prose line under a heading raises `E_DOC` naming the four step keywords
- test — unit test: `But` and `*` raise the same error as any other unknown opener
- test — unit test: a heading with no step raises `E_DOC`
- test — unit test: a `## Scenarios` block with no heading raises `E_DOC`
- test — unit test: two headings sharing a name in one fact raise `E_DOC` naming both lines

### scenarios-parse-or-the-check-fails

- test — unit test: every error reports the line in the fact file, computed from the block offset
- test — unit test: a malformed block makes `compile_docs` exit non-zero
- test — grep assertion: `gherkin` appears in no `Cargo.toml` of the workspace
