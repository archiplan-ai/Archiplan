---
kind: functional
origin: intent
satisfied-by: [Engine, Query]
deferred:
---

# Closure is computed, never stored

A relation declares its direction (`->` or `<->`) and may declare transitivity: `trans`
derives `a -> c` from `a -> b, b -> c`, and symmetrically for undirected chains. Derived
pairs are virtual — only declared edges are stored; the closure is computed at query
time, and only for consumers that opt in — pattern matching, the `types` filter, the
layer probe — while analyses that count edges or node degrees see declared edges alone. A
non-transitive relation matches single declared steps only.

## System Context

Storing derived pairs would turn every edge insert into a cascade and every removal into
a repair — the statefulness whole-model rebuilds exist to avoid. NKP's exclusion patterns
and incidence's type expansion both lean on this same closure rule, so one implementation
answers everyone.

## Satisfy

`Engine` (marks rel types transitive at declaration; walks the closure on demand for
pattern checks). `Query` (the `types` filter and layer probe consume the closure; edge
listings stay declared-only).

- test — semantics::patterns_follow_the_transitive_closure
- test — semantics::non_transitive_relations_match_single_steps_only
- test — semantics::query_types_match_instances_transitively
- test — nkp::exclusion_follows_the_transitive_closure
