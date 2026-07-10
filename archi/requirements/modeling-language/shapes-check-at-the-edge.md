---
kind: functional
origin: intent
satisfied-by: [Engine]
deferred:
---

# Shapes check at the edge

A type's shape restricts what its edges may join: each slot is a pattern — `*` for any
node, an absolute path for exactly that node, `(Type type_of *)` for anything the type
classifies — and every edge is validated against its type's patterns at creation, with
the slot, pattern and offending node in the rejection (E_SHAPE_VIOLATION). Carrier arity
follows the lanes exactly: a lane with a carried slot requires a concrete carried node
matching that slot's pattern (E_CARRIER_REQUIRED), a lane without one rejects any
(E_CARRIER_FORBIDDEN), and a reverse slot requires a directed type. Classified patterns
follow the classifying relation's transitive closure, and lowering lands classifier edges
before the shapes that consult them, so a fresh compile checks every edge against fully
populated patterns — a shape that no longer fits is a compile error at the offending
line, never a lingering nonconformance.

## System Context

Shape conformance plus whole-model rebuilds is the integrity regime: in-place edits could
once erode conformance into drift findings; building whole leaves no room for that state
to exist. The surface's carrier inference (`lanes-carry-the-payload`) fills omitted
exact-node lanes — the engine still sees, and checks, fully explicit statements.

## Satisfy

`Engine` (pattern matching at edge creation, arity per lane, closure-aware classified
patterns).

- test — errors::e_shape_violation_on_ends_and_carrier
- test — errors::e_carrier_required_and_forbidden
- test — bidir_conns::lane_arity_is_checked_per_lane
- test — semantics::patterns_follow_the_transitive_closure
- test — source_e2e::classifier_edges_land_before_shapes_that_consult_them
