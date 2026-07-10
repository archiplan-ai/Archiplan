---
kind: functional
origin: intent
satisfied-by: [Engine]
deferred:
---

# An edge is its statement

Elements are addressed by name and path: a name is unique among siblings, and every node
reference in a statement is an absolute path from the root — there is no session, no
current scope, no relative resolution. Edges carry no name at all: an edge's identity is
the tuple of kind, type, ends with their ports, and carried nodes — so stating an edge is
addressing it. Restating an existing edge is a no-op or a view extension, never a
duplicate, and a reverse carrier is part of the identity like everything else in the
tuple.

## System Context

Everything that must survive edits — requirement references, stressor affects, link spec
refs, version diffs — speaks these addresses; a context-dependent reference would rot the
moment its context moved. Statement objects are plain JSON with a fixed serde shape, so
the identity rule is also what makes whole-batch replays safe
(`define-makes-or-matches` covers the definition half).

## Satisfy

`Engine` (resolves absolute paths only; computes edge identity structurally and folds
restatements onto the existing edge).

- test — semantics::references_are_absolute_only
- test — semantics::edge_identity_is_structural
- test — bidir_conns::rev_carrier_is_part_of_edge_identity
- test — worked_example::statement_objects_round_trip_through_serde
