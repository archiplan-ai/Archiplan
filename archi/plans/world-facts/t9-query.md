---
node: Query
owns: [the-read-envelope-carries-the-conditions]
---

# t9 — Query

the read envelope carries the conditions

## Spec

- `Query`
- `Function type_of Query`
- `Cli.drive consult(->ModelGraph, <-Findings) Query.audit`
- `Engine.slice consult(->ModelGraph, <-Findings) Query.audit`
- `Engine.slice consult(->ModelGraph, <-SubgraphResult) Query.subgraph`

## Inputs

- from t3 — the loaded facts keyed by what they cover

## Outputs

- crates/archi/src/main.rs
- crates/archi/tests/read_e2e.rs

## Stack

- the covering facts attach to a composed slice before it renders
- human and JSON forms both carry them, the JSON under its own key

## Verifications

### the-read-envelope-carries-the-conditions

- test — read_e2e: a slice naming a covered element carries that fact's statement and scenarios
- test — read_e2e: a slice naming an uncovered element carries the slice alone
- test — read_e2e: the JSON envelope carries the facts under their own key
- test — read_e2e: a fact covering two elements of one slice appears once
