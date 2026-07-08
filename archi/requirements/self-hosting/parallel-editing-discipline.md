---
kind: non-functional
origin: stressor(parallel-rounds, union-fuses-charters-silently)
satisfied-by: [Archive, Sessions, Links.Journal]
deferred:
---

# Parallel editing discipline

Two agents hardening the same model on parallel branches need a defined path back to one
lineage: re-minting a save onto the merged history, folding two concurrent rounds into one
record, and a session-open discipline that spans branches rather than working trees.

## System Context

Everything is files in git, so concurrent work is not preventable — only mergeable or not.
First hit by parallel-rounds; mapped store by store in the merge-pressure round, whose derived
requirements sharpen this umbrella: remint-rejoins-the-lineage, merge-deltas-are-reviewable
and the-fold-survives-a-merge (landed — the journal folds concurrent histories, the save
collision has its remint recipe, the live diff reviews a merge before the seal), and
rounds-fold-deliberately. The fold-pressure round settled the last store's boundary
empirically: `merge=union` fuses charter prose into a schema-perfect chimera, so the round
record keeps git's default markers and the tool reads them.

## Satisfy

Store by store, the path back to one lineage is a verb: `Archive.remint` re-mints the
colliding save onto the merged history and re-stamps the round it closes, `Links.Journal`
folds concurrent link histories under its union attribute, the live diff reviews a merge
before any seal, and `Sessions.fold` folds the round record itself — detected by
marker-reading, folded with both charters kept, refused across pins. The merge boundary is
chosen per store — union where replay semantics absorb it, markers where prose identity would
fuse silently — and every merged state lands as one recipe-naming diagnostic, so the
discipline is a printed sequence of verbs, not one writer per repository.

- test — every store's merged state is one recipe-naming diagnostic: manifest markers, stress markers, the open pair
- test — the two-writer lab replays end to end: collide, fold, remint, and `check` closes green with both rounds' records intact
