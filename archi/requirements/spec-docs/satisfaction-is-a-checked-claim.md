---
kind: functional
origin: intent
satisfied-by: [DocsCompiler]
deferred:
---

# Satisfaction is a checked claim

A requirement is satisfied by naming model elements: `satisfied-by` lists absolute model
paths — terms or types, a type covering every term it transitively classifies, the same
expansion stressor affects use — and the `Satisfy` section carries the prose how, closed
by verification bullets (`- test — …`, `- type-level — …`) that say how the claim is
proved. Every satisfied-by path resolves against the live model on every check: rename a
node and forget a requirement that names it, and the build breaks at that requirement's
file and line (`E_MODEL_REF`). A satisfied ancestor satisfies its descendants — check
reports nothing under one — and a satisfaction record with no verification bullets is the
`unverified_satisfaction` finding. Un-satisfying is emptying both halves; there is no
satisfy verb anywhere — the claim is a text edit and a recompile, like every other
mutation.

## System Context

Requirements are living documents outside the version archive: satisfaction tracks the
live tree while stress evidence pins versions — `stress-pins-versions` holds the other
half of that split. The reverse lookup that derives task obligations reads these same
expansions (`tasks-derive-never-retype`).

## Satisfy

`DocsCompiler` (resolves satisfied-by against the current compile, expands type entries
through the closure, applies ancestor transitivity, counts verification bullets).

- test — docs::affects_pin_to_the_sessions_version_while_satisfied_by_tracks_the_live_model
- test — docs::the_worked_tree_checks_out
- test — plans::the_reverse_lookup_matches_terms_types_and_edge_endpoints
