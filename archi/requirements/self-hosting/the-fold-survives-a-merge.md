---
kind: non-functional
origin: stressor(journal-merge-rewrites-truth)
satisfied-by:
deferred:
---

# The fold survives a merge

The journal folds concurrent histories: two branches' events, unioned in either order, reach
one defined live set. Ids minted in parallel do not collide — or a verb re-sequences them;
events that land on a tombstone are defined, not corruption; and whatever repair a merge
demands is a verb that leaves a journal record, never a hand edit of the one file hand edits
are banned from.

## System Context

The journal is sequential-replay truth in a tree that git merges: every branch-parallel pair of
link ops conflicts at the tail, and today the union is corruption or silent judgment loss
decided by line order. Pressed for real by merge-pressure.

## Satisfy
