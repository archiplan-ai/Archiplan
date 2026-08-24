---
node: DocsCompiler
owns: [coverage-reaches-down-the-graph, an-internal-element-says-so]
facts: [a-design-written-apart-from-the-code-falls-behind-it@4bc5db, why-a-design-was-chosen-lives-in-one-person-s-memory@97f511]
---

# t1 — DocsCompiler

Data leaves the coverage question and .worldignore names what is internal

## Spec

- `DocsCompiler`
- `Function type_of DocsCompiler`
- `Cli.drive consult(->ModelGraph, <-Diagnostics) DocsCompiler.compile_docs`
- `Incidence.pin_model recall(<-ModelGraph) DocsCompiler.serve_pin`
- `Links.pin_spec recall(<-ModelGraph) DocsCompiler.serve_pin`
- `Links.pin_spec recall(<-WorldDoc) DocsCompiler.serve_world`
- `Planner.load_world recall(<-WorldDoc) DocsCompiler.serve_world`
- `Planner.pin_model recall(<-ModelGraph) DocsCompiler.serve_pin`
- `Query.read_world recall(<-WorldDoc) DocsCompiler.serve_world`

## Inputs


## Outputs

- crates/archi/src/docs/world_check.rs
- crates/archi/src/docs/mod.rs
- crates/archi/tests/check_e2e.rs

## Stack

- the covered set skips every element the model classifies as `Data`, by its type and not by a list
- `archi/world/.worldignore` is read beside the world; one line is `<element path> — <reason>`
- the scenario digest moves here, beside the parsed block, so the plan and the link read one function

## Verifications

### coverage-reaches-down-the-graph

- test — a Data-classified element is never reported, covered or not
- test — an element named in `.worldignore` is not reported
- test — an element reachable from a covered element, and a child of one, are still not reported
- test — check_e2e: a world of one fact reports a list that names no Data element

### an-internal-element-says-so

- test — an entry naming a model element suppresses its `world_unreached`
- test — an entry naming nothing raises a located error at its line
- test — an entry with no reason raises a located error at its line
- test — a Data-classified entry raises a located error, because its type already excludes it
- test — a tree with no `.worldignore` behaves exactly as it does today
- test — check_e2e: with every element conditioned, Data or declared, the report is empty
