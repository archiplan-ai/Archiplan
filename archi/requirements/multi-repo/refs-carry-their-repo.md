---
kind: functional
origin: intent
satisfied-by: [Links]
deferred:
---

# Refs carry their repo

A code ref names the member its file lives in — `member//path#symbol` — and the
unqualified form keeps meaning the project's own repository, so every ref ever
written, on the command line or in the journal, keeps its meaning unchanged. Birth
records, projections, findings and reports render member-qualified paths; the
journal stays append-only with no migration.

## System Context

Anchors, spans and scan keys are today bare root-relative paths; across several
roots a bare path is ambiguous. The journal is the links' truth and append-only —
a qualifier must arrive as an optional field old events simply lack, never as a
rewrite.

## Satisfy

`Links` (anchors and spans carry an optional member, absent meaning home; parse, render and the
journal agree on the `member//file#symbol` surface; old events replay unchanged).

- test — `backend//src/api.rs#serve` round-trips through link add, ls and the journal event
- test — a journal written before members replays with every anchor resolving home
- test — an unqualified ref added today folds identically to its home-qualified twin
