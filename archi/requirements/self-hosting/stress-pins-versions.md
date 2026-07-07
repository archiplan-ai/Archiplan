---
kind: non-functional
origin: intent
satisfied-by: [DocsCompiler, Archive]
deferred:
---

# Stress pins versions

A stress session presses on exactly one archived version, and every stressor's affects resolve
against that version's reconstructed model — however the live tree evolves meanwhile. Evidence
must not move: a later rename can never orphan an affects list, and old sessions stay
re-analyzable forever.

## System Context

Stress rounds and model edits interleave in the same working tree; the session must stay
coherent while the answers land.

## Satisfy

`DocsCompiler` validates each open session's affects against the model `Archive` reconstructs
for the session's pinned id, not against the live tree.

- test — pin a session, rename an affected node in the live tree; check stays green until the pin moves
- test — the incidence run over a closed session expands affects against its pinned version's terms
