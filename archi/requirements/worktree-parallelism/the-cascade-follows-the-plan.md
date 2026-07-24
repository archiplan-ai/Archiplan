---
kind: functional
origin: intent
satisfied-by: []
deferred:
---

# The cascade follows the plan

A mint cascades to the member repositories the work names — `--repos`,
derived by the agent from the spec and plan (task outputs, refs, links): the
same branch in every participating repo, one worktree per physical
repository, validated whole before anything is created and rolled back whole
on partial failure. A re-mint of the same slug extends the seat, never
recreates it.

## System Context

Each member branch grows from a base the pinned version's recorded baseline
proves reachable; an unrecorded, unfetched or unreachable baseline refuses
with candidate branches and the `--base member=branch` escape — a question,
never a guess (worktrees-mint-on-demand). Members sharing one git repository
share one worktree; the registry records every member seat
(the-registry-binds-the-worktree).

## Satisfy
