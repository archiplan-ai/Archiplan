---
kind: non-functional
origin: stressor(journal-merge-rewrites-truth)
satisfied-by: [Links.Journal]
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

`Links.Journal` mints collision-free ids — `l<seq>-<hash>`, the suffix derived from the link's
content and mint moment — and folds concurrent histories: events that land on a tombstone or
replay an identical line are absorbed and surfaced as notes, not corruption; only events naming
ids the journal has never seen remain corrupt. A union merge attribute ships beside the journal
so branch merges concatenate instead of conflicting.

- test — two branches' adds union-merge and both links fold live, in either order
- test — retire∥repin unions fold to the same live set in both orders, the absorbed event surfaced as a note
- test — an event naming an id the journal never minted still refuses the fold
