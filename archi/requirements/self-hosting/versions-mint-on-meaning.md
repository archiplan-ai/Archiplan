---
kind: functional
origin: intent
satisfied-by: [Archive, Renderer]
deferred:
---

# Versions mint on meaning

A version exists exactly when the model changed. Comment, formatting and file-organization churn
never mints a version; a save against an unchanged model refuses.

## System Context

Versions are the anchors everything else pins to — stress sessions, plans, links. Cheap versions
would drown the anchors in noise.

## Satisfy

`Renderer` canonicalizes the compiled model — comments and layout stripped, statements in
lowering order — and `Archive` identifies a version by the sha256 of those bytes, refusing a
save whose hash equals the latest entry.

- test — reformat and reshuffle sources without a semantic edit; version save refuses with nothing to save
- test — canonical::scope_sources_slice_the_render_per_root grounds the derived per-scope hashes
