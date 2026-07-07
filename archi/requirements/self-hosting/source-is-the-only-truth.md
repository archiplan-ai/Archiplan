---
kind: non-functional
origin: intent
satisfied-by: [SourceTree, Compiler]
deferred:
---

# Source is the only truth

The `.arch` source tree is the only persistence of the model: every run compiles the project
fresh, and there is no mutation vocabulary anywhere — not in source, not in the statement layer.
A change to the model is a text edit and a recompile, and the diff is the change record.

## System Context

A plain git repository is the substrate — no daemon, no database, no build artifact to keep in
sync. Agents and humans edit the same files.

## Satisfy

`SourceTree` is the single store every component reads from and writes to; `Compiler` recompiles
it from scratch on every verb, so no state can exist that the source does not encode. Dumps
render as valid surface syntax and replay to the identical model.

- test — semantics::mutation_vocabulary_does_not_exist: replaying retired mutation verbs is E_PARSE
- test — source_e2e::dumps_are_valid_surface_and_round_trip, source_e2e::the_compiled_batch_replays_into_the_same_model
