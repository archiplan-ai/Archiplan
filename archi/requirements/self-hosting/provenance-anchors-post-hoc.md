---
kind: functional
origin: stressor(dirty-tree-bootstrap)
satisfied-by: [Archive]
deferred:
---

# Provenance anchors post hoc

A version minted on a dirty tree can gain commit provenance after the fact, under exactly the
guarantee save-time recording gives: the working tree is clean and its render hashes to the
version being anchored, so the recorded commit provably contains the render's sources. Recorded
provenance is a birth fact — anchoring a version that has one reports it and never rewrites it.
Whenever the audit lacks a delta source, it names this recovery path instead of only naming its
own blindness.

## System Context

Adoption saves before the first commit by construction — a bootstrap writes the model and the
archive into an uncommitted tree — while the audit's default delta source is the latest version's
commit provenance.

## Satisfy

`Archive` (the `anchor` port; `satisfied-by` cannot yet pin ports —
`issues/satisfied-by-cannot-name-ports-or-edges.md`) resolves the entry whose sealed hash matches
the live render, refuses dirty trees and unmatched renders loudly, records HEAD once, and treats
re-anchoring as a reporting no-op. `Cli.version` exposes it as `archi version anchor`; the
sweep's no-delta-source note names the commit-then-anchor recovery.

- test — anchor_records_provenance_post_hoc: no repo and dirty tree refused; clean matching tree records HEAD; re-anchor is a no-op; provenance survives HEAD moving on, never rewritten
- test — the audit consumes the anchor: adopt in a scratch repo, commit, anchor — the no-delta-source note disappears and a post-anchor code delta is attributed unaccounted
