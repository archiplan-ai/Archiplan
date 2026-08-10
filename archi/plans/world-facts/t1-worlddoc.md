---
node: WorldDoc
owns: [a-world-fact-carries-its-scenarios, the-header-points-three-ways, a-source-may-lie-outside-the-tree]
---

# t1 — WorldDoc

the world-fact record: frontmatter, headings, the three lists

## Spec

- `WorldDoc`
- `Data type_of WorldDoc`

## Inputs


## Outputs

- crates/archi/src/docs/world.rs
- crates/archi/src/docs/schema.rs
- crates/archi/src/docs/mod.rs

## Stack

- serde for the frontmatter, alongside the requirement and stressor schemas in `schema.rs`
- `docs/world.rs` — the record type, its heading order and the two `sources` entry forms
- reuse the located-error type the requirement and stressor schemas already emit

## Verifications

### a-world-fact-carries-its-scenarios

- test — world_e2e: a file with name, paragraph, `What kills this` and one scenario parses clean
- test — world_e2e: a missing paragraph, a missing killer and an empty `Scenarios` each raise E_DOC at their line
- test — world_e2e: a file with no `Open questions` heading parses, and one with an empty heading parses
- test — world_e2e: an unknown heading is kept and raises nothing

### the-header-points-three-ways

- test — world_e2e: an unresolved entry in `covers`, in `sources` and in `uses` each raise a located error
- test — world_e2e: an empty `sources` parses and the record reads as ungrounded
- test — world_e2e: a fourth frontmatter key raises a located error

### a-source-may-lie-outside-the-tree

- test — world_e2e: a schemeless entry resolves as a tree path and a missing file raises a located error
- test — world_e2e: a schemed entry parses with no filesystem access
- test — world_e2e: a malformed schemed entry raises a located error
- test — world_e2e: a record mixing both forms parses
