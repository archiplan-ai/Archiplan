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
requirements sharpen this umbrella: remint-rejoins-the-lineage, merge-deltas-are-reviewable,
the-fold-survives-a-merge, rounds-fold-deliberately. The discipline itself is specified in
requirements/multiplayer.md; until the verbs land, the operating rule stays one writer per
repository.

## Satisfy
