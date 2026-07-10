---
kind: functional
origin: intent
satisfied-by: [Nkp]
deferred:
---

# The landscape is a slice

NKP never runs on the raw graph. The landscape drops the preset's scaffolding entirely,
then disqualifies nodes by exclusion pattern: a pattern is edge-shaped over a named
relation — `<source> <rel> <target>` with exactly one `_` slot — excluding any node that
can fill the `_` so the relation holds, following the relation's transitive closure when
it has one. The defaults mirror the default preset: `_ type_of *` drops the epistemic
layer, `Data type_of _` drops data as endpoints — while carriers survive, because a
connection carrying a data node still couples its endpoints; only data as an endpoint
drops an edge. A pattern naming an unknown rel or node matches nothing and warns
(`UNKNOWN_EXCLUDE_REF`) instead of failing. Scope picks the frame: recursive (the
default) spans all scopes and folds delegation applications — a node realizing its
parent's port merges into the parent, its couplings re-attaching there; top sees
top-level nodes only; `--scope <path>` reads one node's direct children without folding.
`--only` narrows which rel/conn types count as coupling at all.

## System Context

The slice encodes what "a component" means — `terms-and-types-are-layers` supplies the
layer boundary — and a landscape that counted types or payloads would report the
vocabulary's size, not the design's coupling.

## Satisfy

`Nkp` (slices before measuring: preset out, patterns applied through the closure, scopes
folded or cut, edge types filtered).

- test — nkp::default_slice_keeps_behavior_drops_data_types_and_preset
- test — nkp::exclusion_follows_the_transitive_closure
- test — nkp::custom_patterns_and_unknown_refs
- test — nkp::folding_and_scope_modes
- test — nkp::only_edge_types_narrows_coupling
