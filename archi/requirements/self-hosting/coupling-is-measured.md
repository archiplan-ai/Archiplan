---
kind: functional
origin: intent
satisfied-by: [Nkp, Incidence]
deferred:
---

# Coupling is measured

The architecture's coupling is a number you can watch, not a feeling: the landscape read says
where the model sits between order and chaos and which nodes are epistatic hubs; the incidence
read pivots a stress round into the stressor × component matrix and its typed findings — hidden
coupling, hotspots, compound vulnerabilities, under-stressed corners.

## System Context

Both analyses read the compiled model only — no instrumentation, no runtime traces; incidence
additionally reads the session docs and the archive.

## Satisfy

`Nkp` computes K/P metrics, regime, hotspots and corridors over the epistatic slice; `Incidence`
joins stressor affects — expanded against their pinned versions — with the invariant surface into
the matrix and findings, firing automatically on the save that closes a session.

- test — nkp::hub_hotspot_and_corridors, nkp::default_slice_keeps_behavior_drops_data_types_and_preset
- test — incidence::response_similarity_splits_on_declared_connectivity, incidence::compound_vulnerabilities_take_two_surviving_partial_covers
