---
kind: functional
origin: intent
satisfied-by: [Archive, Members]
deferred:
---

# Provenance goes per member

A version's commit provenance becomes a set: at save, each mapped member whose
tree is clean contributes its baseline — the commit its code stood at when the
architecture was agreed — and the audit's delta source is each member's own
baseline. A member without one gets its own recovery note naming the member; the
other members' audits proceed. Baselines stay provenance, never a dependency.

## System Context

Today one commit field anchors the whole audit, and it certifies the render's
sources — the spec. When the spec repository holds no code, that field is true but
vacuous, and the audit is structurally blind. Per-member baselines keep the audit's
question — what moved since the architecture was agreed — answerable per tree,
degrading per member instead of globally.

## Satisfy

`Archive` (save writes one baseline per clean mapped member into the manifest entry and reports
the omitted; anchor extends per member under the clean-tree guarantee), `Members` (the resolved
set a save baselines against).

- test — a save with one clean and one dirty member records exactly the clean baseline and names the omission
- test — `version anchor --repo <member>` records a missing baseline post hoc and re-anchoring reports it unchanged
- test — a version entry without the baseline table replays: home provenance semantics untouched
