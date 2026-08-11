---
node: Archive
owns: [the-save-refuses-an-unconditioned-element]
---

# t3 — Archive

the save refuses an element no condition reaches

## Spec

- `Archive`
- `Storage type_of Archive`
- `Cli.drive consult(->ModelGraph, <-Report) Archive.anchor`
- `Cli.drive consult(->ModelGraph, <-Report) Archive.remint`
- `Cli.drive consult(->ModelGraph, <-Report) Archive.save`
- `DocMint.pin recall(<-CanonicalRender) Archive.serve`
- `DocsCompiler.pin_model recall(<-CanonicalRender) Archive.serve`
- `Seats.locate recall(<-MemberBaseline) Archive.serve`

## Inputs


## Outputs

- crates/archi/src/versions.rs
- crates/archi/src/docs/world_check.rs
- crates/archi/tests/version_e2e.rs
- archi/world/.worldignore

## Stack

- the unreached set is the one `world_check` already computes for its finding; the save asks the same function and refuses instead of reporting
- the refusal names every element and both exits, in the shape the other refusals use
- this repository carries thirteen unreached elements today, so the task also writes its own `.worldignore`: the gate lands with the tree that satisfies it, or no version saves after it

## Verifications

### the-save-refuses-an-unconditioned-element

- test — version_e2e: a tree with one fact and one unreached element refuses `version save`, naming the element
- test — version_e2e: the refusal names both exits — cover it, or declare it internal
- test — version_e2e: covering the element clears the refusal and the save proceeds
- test — version_e2e: declaring it in `.worldignore` clears the refusal as well
- test — unit test in `world_check.rs`: a `Data`-classified element never enters the set the save reads
- test — version_e2e: a tree with no world facts saves byte-identically to before
- test — unit test: `check`'s finding and the save's refusal read the same set, computed once
- test — e2e: `archi version save` on this repository's own tree proceeds, because its `.worldignore` covers what no fact reaches
