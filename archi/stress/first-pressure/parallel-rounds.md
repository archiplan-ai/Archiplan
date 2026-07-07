---
affects: [DocsCompiler, SourceTree]
outcome: breaking
---

# Parallel rounds

Two agents on two branches each open a stress session against v0001, answer their stressors and
save: both branches now hold an open-session stamp, a v0002 manifest entry and a v0002 patch
file — and then the branches merge.

## Attractor

The merged tree carries two open sessions and colliding version ids; whichever agent merges
second inherits a broken archive and a session state nobody authored, and the repair is
improvised under pressure.

## Resolution

Bends. The collisions are loud by design — two open sessions are E_SESSION, colliding ids break
the dense-sequence check and conflict as ordinary git merges — but loud is not the same as
answered: nothing helps the second agent re-mint a save onto the merged lineage or fold two
rounds into one record. Concurrency discipline is a spec stub. Derived: parallel-editing-discipline,
deferred until multiplayer lands; until then the discipline is one writer per repository.
