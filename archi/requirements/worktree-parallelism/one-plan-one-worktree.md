---
kind: non-functional
origin: intent
satisfied-by: []
deferred:
---

# One plan, one worktree

At most one live worktree mutates a plan. A mutating plan verb from any other checkout refuses
and names the owning path. Handoff is sequential: the branch travels, and the next worktree is
minted wherever it lands.

## System Context

The invariant is a registry lookup (the-registry-binds-the-worktree), backed by git's own rule
that one branch checks out in at most one worktree. It closes the writer race
parallel-editing-discipline left open: PlanFile lifecycle and wave snapshots have no fold
verb, so they must never see two concurrent writers in the first place.

## Satisfy
