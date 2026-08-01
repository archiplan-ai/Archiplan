---
kind: functional
origin: intent
satisfied-by: [Seats.Mint, Members, Archive]
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

`Seats.Mint` cascades to the members `--repos` names: resolution through `Members`, each
branch grown from the baseline `Archive` recorded, the whole cascade validated before
anything is created and rolled back whole on partial failure; a re-mint extends the seat and
resolves members from the invoked root — the seat's own manifest and overlay mid-unit.

- test — the cascade mints member worktrees and the seat overlay (`the_cascade_mints_member_worktrees_and_the_overlay`)
- test — an off-branch baseline refuses with candidate branches and the `--base` escape (`a_baseline_off_the_branch_refuses_with_the_base_escape`)
- test — a missing baseline names both repairs (`a_missing_baseline_names_both_repairs`)
- test — a partial cascade rolls back whole (`a_partial_cascade_rolls_back_whole`)
- test — a seat extension resolves members from the seat, not the primary (`an_extension_resolves_members_from_the_worktree`)
