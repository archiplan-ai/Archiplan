---
kind: non-functional
origin: stressor(parallel-rounds)
satisfied-by:
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
rounds-fold-deliberately (open). The discipline is specified in requirements/multiplayer.md;
what still needs one writer at a time is the round record itself — concurrent sessions merge
detectably but fold only by hand.

## Satisfy
