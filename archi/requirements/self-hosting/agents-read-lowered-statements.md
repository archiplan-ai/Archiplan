---
kind: functional
origin: intent
satisfied-by: [Engine, Query]
deferred:
---

# Agents read lowered statements

An agent's read surface is the lowered statement layer: batches of read statements in, structured
results out, over files, stdin or flags — and against any archived version, which is how an agent
grounds itself on a plan's pin. The envelope is read-only; a write smuggled into it is rejected,
not applied.

## System Context

Agents cannot hold a long-lived process open; every read rides one CLI invocation.

## Satisfy

`Engine` answers the envelope and rejects non-read statements with E_BAD_REQUEST; `Query` slices
the model by composed filters — scope, type, kind, view, carrier, edge-type — with validated
filter names.

- test — read_e2e::the_envelope_reads_batches_and_refuses_writes
- test — read_e2e::query_composes_filters_and_reads_sealed_versions
