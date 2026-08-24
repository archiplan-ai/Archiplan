---
node: DocsCompiler
owns: [the-world-holds-four-layers, a-source-is-reachable-and-lives-in-the-world]
facts: [a-design-written-apart-from-the-code-falls-behind-it@e3b0c4, why-a-design-was-chosen-lives-in-one-person-s-memory@e3b0c4]
---

# t1 — DocsCompiler

the four folders and the rule that a source lives inside the world

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

- discovery walks `facts/` for the strict schema and `hypotheses/`, `notes/` for a name and prose
- `resources/` is listed for resolution and never parsed
- a `sources` entry resolves against the world's own files; a path outside it and a URI scheme are both located errors

## Verifications

### the-world-holds-four-layers

- test — unit test in `world_check.rs`: a file under `facts/` missing its killer or its scenarios is a located error
- test — unit test: a file under `notes/` with a name and one paragraph passes with no diagnostic
- test — unit test: a file under `hypotheses/` with a name and one paragraph passes with no diagnostic
- test — unit test: a file under `resources/` is never parsed and never reported, whatever it holds
- test — unit test: a tree holding none of the four folders reports nothing, as today
- test — check_e2e: a tree with all four folders exits 0 and counts only the facts

### a-source-is-reachable-and-lives-in-the-world

- test — unit test: a source naming a file under `notes/`, `hypotheses/` or `resources/` resolves
- test — unit test: a source naming a file that does not exist raises a located error
- test — unit test: a source naming a path outside `archi/world/` raises a located error whose text says why
- test — unit test: a source carrying a URI scheme raises that same error
- test — unit test: an empty `sources` passes and the fact reports `world_ungrounded`
- test — check_e2e: this tree's four facts carry no source and each reports ungrounded
