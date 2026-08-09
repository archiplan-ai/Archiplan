---
affects: [Seats.Mint]
outcome: surviving
---

# the member checkout is busy

Cascade to a member whose main checkout stands on another branch, holds
uncommitted work, or sits mid-rebase.

## Attractor

A refresh that pulls would move the wrong branch, create a merge commit
in someone's tree, or refuse outright — a mutation of a workspace nobody
asked us to touch.

## Resolution

Held by the choice of verb: the refresh fetches and never pulls, so it
writes only remote-tracking refs and leaves every working tree exactly
as it was. The branch point is then computed from refs alone.
