---
affects: [Seats.Mint]
outcome: breaking
---

# the local branch is ahead of its remote

Commit to the base branch and do not push it — a landed unit, a manifest
edit, an hour of local work. Then mint the next seat.

## Attractor

Branching from the remote ref silently drops that work: the new seat
starts from a world where it never happened, and the operator meets it
again as a conflict days later. The fix for staleness becomes a new way
to lose commits.

## Resolution

The branch point follows the divergence, and the safe side is always
local: behind the remote takes the remote ref, ahead of it or diverged
from it takes the local branch, and both counts are named in the report.
Derived `the-branch-point-follows-the-divergence`.
