---
affects: [Links.Journal, LinkEvent]
outcome: breaking
---

# Journal merge rewrites truth

Two writers touch links between saves: each `link add` mints the same next id on its own
branch; one retires `l0001` while the other repins it. The branches merge, and the operator
resolves the journal's tail conflict the only way git offers — by hand.

## Attractor

The append-only truth stops folding: the live set depends on merge order, or the subsystem
refuses to answer at all.

## Resolution

Broke, hard. Every branch-parallel pair of link ops conflicts at the journal tail, so the one
file the ground rules say never to hand-edit is precisely the file every merge forces into an
editor. Keeping both writers' work — the union — is corruption either way: duplicate adds fold
to `journal corrupt: 'l0428' is added twice` and every link verb exits 1; the retire∥repin race
folds by line order — one order silently absorbs the repin into the tombstone (a judgment lost
without trace), the other is `journal corrupt: 'repin' names 'l0001', which is not a live
link` and the subsystem is down. The resolver chooses between silent loss and corruption
without knowing there is a choice. No verb renumbers an id, re-sequences a segment, or folds
two journals; the repair is surgery on a JSON line or discarding a colleague's judgment.
Sequential-replay truth does not yet admit concurrent history.
Derived: the-fold-survives-a-merge.
