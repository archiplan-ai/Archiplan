---
kind: non-functional
origin: intent
satisfied-by: [Seats.Registry, Seats.Guard]
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

`Seats.Registry` records which worktree carries which plan; `Seats.Guard` refuses a mutating
plan verb from any other checkout, naming the owning path — backed by git's one-branch-one-
checkout rule.

- test — a second checkout mutating a bound plan refuses with the owner's path (`an_unbound_checkout_mints_the_worktree_and_the_work_proceeds`)
