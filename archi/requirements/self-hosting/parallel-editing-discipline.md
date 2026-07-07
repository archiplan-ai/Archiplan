---
kind: non-functional
origin: stressor(parallel-rounds)
satisfied-by:
deferred: multiplayer is a spec stub; until it lands the discipline is one writer per repository — branch-parallel saves and sessions merge loudly (E_SESSION, dense-id conflicts) and the second writer re-mints by hand
---

# Parallel editing discipline

Two agents hardening the same model on parallel branches need a defined path back to one
lineage: re-minting a save onto the merged history, folding two concurrent rounds into one
record, and a session-open discipline that spans branches rather than working trees.

## System Context

Everything is files in git, so concurrent work is not preventable — only mergeable or not;
today's checks make collisions loud but leave the repair to the operator.

## Satisfy
