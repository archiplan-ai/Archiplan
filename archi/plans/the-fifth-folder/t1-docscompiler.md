---
node: DocsCompiler
owns: [the-world-holds-four-layers]
facts: [a-design-written-apart-from-the-code-falls-behind-it@c8d7cc, what-the-work-is-really-like-lives-in-the-head-of-whoever-does-it@893bd1, why-a-design-was-chosen-lives-in-one-person-s-memory@4fecb2]
---

# t1 — DocsCompiler

a file under a folder that is not one of the four is refused by path

## Spec

- `DocsCompiler`
- `Function type_of DocsCompiler`
- `Cli.drive consult(->ModelGraph, <-Diagnostics) DocsCompiler.compile_docs`
- `Links.pin_spec recall(<-WorldDoc) DocsCompiler.serve_world`
- `Planner.load_world recall(<-WorldDoc) DocsCompiler.serve_world`
- `Query.read_world recall(<-WorldDoc) DocsCompiler.serve_world`

## Inputs

## Outputs

- crates/archi/src/docs/world_check.rs
- crates/archi/tests/world_e2e.rs

## Stack

- `discover` at `crates/archi/src/docs/world_check.rs:190` filters `sorted_entries(&base)` by `is_md`, which sees files directly under `archi/world/` and never opens a folder — the walk has to descend
- every `.md` under a directory that is not one of `LAYERS` raises the E_PLACEMENT the loose file already raises, at any depth below `archi/world/`
- one message built once from `LAYERS`, read by both arms, so the loose file and the foreign folder can never name different layers
- `.worldignore` is no document and stays out of the walk, as it already is
- a directory holding no `.md` says nothing: the requirement locates a file, and there is no file to locate

## Verifications

### the-world-holds-four-layers

- test — `archi check` on a tree holding `archi/world/attic/thing.md` exits 1 with an E_PLACEMENT located at that path, naming all four layers
- test — the same for `archi/world/attic/deep/thing.md`, so the walk is not one level deep
- test — the message is the one a loose `archi/world/stray.md` raises, differing only in the file it names
- test — the four layers keep passing untouched, and a `.txt` under `resources/` is still never parsed and never reported
- test — an empty folder outside the four raises nothing
- test — a tree with no `archi/world/` at all still says nothing
