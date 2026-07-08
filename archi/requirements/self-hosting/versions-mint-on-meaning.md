---
kind: functional
origin: intent
satisfied-by: [Archive, Renderer]
deferred:
---

# Versions mint on meaning

A version exists exactly when the model changed. Comment, formatting and file-organization churn
never mints a version; a save against an unchanged model mints nothing — it still closes an open
stress round and succeeds (`unchanged-saves-close-rounds`), but the archive gains no entry.

## System Context

Versions are the anchors everything else pins to — stress sessions, plans, links. Cheap versions
would drown the anchors in noise.

## Satisfy

`Renderer` canonicalizes the compiled model — comments and layout stripped, statements in
lowering order — and `Archive` identifies a version by the sha256 of those bytes, declining to
mint when the hash equals the latest entry.

- test — reformat and reshuffle sources without a semantic edit; version save mints nothing and reports the model unchanged
- test — canonical::scope_sources_slice_the_render_per_root grounds the derived per-scope hashes
