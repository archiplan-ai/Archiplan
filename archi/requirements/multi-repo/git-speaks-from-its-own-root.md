---
kind: functional
origin: stressor(the-monorepo-that-was-already-broken)
satisfied-by: [Links, Archive, Members]
deferred:
---

# Git speaks from its own root

Every git consultation — provenance, cleanliness, `--since` diffs, audit hunks — resolves the
repository's actual top level and rebases its output into the consulted member's frame before
any comparison. Home is a member like any other, so a project rooted below its git root stops
mismatching silently; paths that leave the member's frame after rebasing are dropped with a
note, never compared raw.

## System Context

Git reports paths relative to its own root; archi compares them against project-root-relative
paths. The two roots coincide only in the unnested single-repo shape, and nothing today checks
that they do — the comparison just quietly misses. Nesting is a blessed shape and members
multiply the divergent cases, so the rebase must sit under every git read, not beside one.

## Satisfy

`Members` (resolution carries each member's git top level beside its root), `Links` (diff and
audit scans consume rebased paths), `Archive` (cleanliness and provenance judged in the member's
frame).

- test — a project rooted below its git root audits and verifies with correctly rebased paths
- test — a member whose checkout sits below a bigger repository's root rebases the same way
- test — a diff path outside the member's frame is dropped with a note, not compared raw
