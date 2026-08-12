---
node: DocsCompiler
owns: [a-fact-may-stand-before-the-model-does, an-ungrounded-fact-says-so, the-fact-speaks-the-world-and-check-says-when-it-does-not, one-fact-reports-one-finding, the-world-reports-what-stands-in-the-air, the-check-counts-the-world, coverage-reaches-down-the-graph]
---

# t3 — DocsCompiler

load the world, cross-check it, report its states

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

- from t1 — the parsed world record and its located errors
- from t2 — the parsed scenario set per fact, or the grammar's located error

## Outputs

- crates/archi/src/docs/mod.rs
- crates/archi/src/docs/world_check.rs

## Stack

- `covers` resolves against the compiled model the docs pass already holds
- the covered set is forward reachability over connection edges plus containment
- `serve_world` hands the loaded facts to the planner, the links and the query surface

## Verifications

### a-fact-may-stand-before-the-model-does

- test — a fact with empty `covers` passes check and reports `world_uncovered`
- test — a fact with empty `covers` and one scenario is not an orphan
- test — a fact with no inbound `uses` and no scenarios reports `world_orphan`

### an-ungrounded-fact-says-so

- test — a fact with empty `sources` and no `uses` reports `world_ungrounded`
- test — a fact with any source entry reports nothing

### the-fact-speaks-the-world-and-check-says-when-it-does-not

- test — a conditioning paragraph naming a model element reports `world_speaks_the_model` with the element
- test — the same name inside a scenario step reports nothing

### one-fact-reports-one-finding

- test — a fact holding three states reports one `world_state` line naming all three
- test — a healthy fact reports nothing

### the-world-reports-what-stands-in-the-air

- test — a `uses` cycle of two facts raises a located error naming both
- test — a chain of four facts reports `world_chain_deep` and a chain of three does not

### the-check-counts-the-world

- test — a tree with seven facts, two without sources, prints both numbers
- test — a tree with no world facts prints no line

### coverage-reaches-down-the-graph

- test — an element reachable from a covered element is not reported
- test — a child of a covered element is not reported
- test — an element nothing reaches reports `world_unreached` by path
- test — a tree with no world facts reports nothing about coverage
