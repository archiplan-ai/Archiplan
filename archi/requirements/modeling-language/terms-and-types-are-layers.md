---
kind: functional
origin: intent
satisfied-by: [Query]
deferred:
---

# Terms and types are layers

Every node sits in one of two layers, told apart by the stdlib classifier alone: a node
that never stands on the left of `type_of` is a term — the epistatic layer, the concrete
structure the system is actually built from — and a node that classifies at least once is
a type — the epistemic layer, knowledge about that structure. The split is derived from
declared edges, never stored, so it moves with the model: stressor affects press terms,
NKP's landscape drops the epistemic layer by default, and a `satisfied-by` or `affects`
entry naming a type expands to the terms it transitively classifies.

## System Context

One relation carrying the ontology is what lets the preset stay swappable —
`one-ontology-everywhere` demands the classifier's exact shape — and every analysis agree
on what "a component" means without a second taxonomy to keep in sync.

## Satisfy

`Query` (the layer probe reads the `type_of` closure off the compiled graph; filters,
analyses and expansions consult it rather than re-deriving).

- test — worked_example::layers_follow_type_of
- test — nkp::default_slice_keeps_behavior_drops_data_types_and_preset
- test — incidence::a_type_expands_to_the_terms_it_transitively_classifies
