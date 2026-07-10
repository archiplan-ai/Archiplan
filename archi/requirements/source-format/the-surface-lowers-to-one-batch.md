---
kind: functional
origin: intent
satisfied-by: [Compiler.Lower, Renderer]
deferred:
---

# The surface lowers to one batch

A project compiles to exactly one statement batch, executed on a fresh workspace holding
the manifest's preset — the surface adds no second door to the model. Dumps render in the
surface syntax: creation statements are valid source, so a dump pastes back into a module
and recompiles to the identical model. The caveat is inherent and stays: a
statement-built model may use reserved words as names or lean on use-created ports — such
models replay via the statement layer but do not round-trip through source. Reads have no
surface form; the source is state, not a transcript.

## System Context

Sugar over the statement layer is the format's founding constraint —
`one-semantic-authority` holds the semantics, `renders-are-layout-blind` and
`deterministic-lowering` own the batch's ordering guarantees. Round-trip fidelity is what
lets the archive store canonical renders and still call them source
(`versions-mint-on-meaning` hashes them).

## Satisfy

`Compiler.Lower` (every construct lowers to ordinary creation statements, one batch per
project). `Renderer` (dumps emit valid surface syntax that recompiles to the same model).

- test — source_e2e::dumps_are_valid_surface_and_round_trip
- test — source_e2e::the_compiled_batch_replays_into_the_same_model
- test — worked_example::dump_round_trips_idempotently
- test — declared_ports::dump_with_declared_ports_replays_idempotently, bidir_conns::dump_with_rev_carriers_replays_idempotently
