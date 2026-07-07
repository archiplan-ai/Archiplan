---
kind: non-functional
origin: intent
satisfied-by: [Compiler.Lower]
deferred:
---

# Deterministic lowering

Identical sources lower to a bit-identical statement batch, whatever the filesystem iteration
order or authoring order. Everything downstream — version hashes, scope hashes, semantic diffs —
inherits its meaning from this guarantee.

## System Context

Versioning hashes canonical renders; two machines must mint the same hash for the same model or
the archive cannot be shared.

## Satisfy

`Compiler.Lower` emits the batch in a fixed order — nodes in path order, views and types in name
order, rel types topologically, edges in authoring order — independent of discovery order.

- test — source_e2e::compilation_is_deterministic_under_source_order permutes sources and compares batches
- test — canonical::render_source_is_deterministic_and_a_fixed_point
