---
affects: [Incidence]
outcome: surviving
---

# Unclassified terms stay loud

A project runs the core preset — no ontology, no `Data` node anywhere — or classifies only half
its terms while the model grows faster than its `type_of` edges.

## Attractor

The filter's boundary drifts from "classified under Data" to "not classified as behavior": every
unclassified term gets muted, a bare-preset project's under-stressed sweep goes silent entirely,
and the finding that exists to expose blind spots develops one of its own — the least-modeled
corners of the system are exactly the ones the sweep stops naming.

## Resolution

Holds on v0004 (no filter yet, everything loud) and fences the fix from the other side: the
boundary is NKP's own — drop exactly the `type_of` closure of a term named `Data`, keep
everything else. No `Data` in the preset means the closure is empty and the sweep is complete;
an unclassified term is never muted, by construction rather than by policy. Pinned by a
regression: a core-preset model emits under-stressed findings for every zero column, filter or
no filter.
