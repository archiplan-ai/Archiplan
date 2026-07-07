---
kind: non-functional
origin: intent
satisfied-by: [Preset]
deferred:
---

# One ontology everywhere

Every model starts from a pinned preset — the stdlib of nodes and the type_of classifier — so
agents describe systems in a shared vocabulary instead of inventing ontology on the fly. Preset
elements are substrate: referencable and extensible, but never redefined, never dumped, never
reported against.

## System Context

The manifest pins the preset by name or path; analyses key their default filters off preset
membership.

## Satisfy

`Preset` loads the named creation batch into every fresh workspace before user statements and
rejects a preset that omits or bends the type_of classifier.

- test — preset::default_ontology_loads_sealed_and_queryable
- test — preset::preset_validation rejects a preset without the exact type_of shape
