---
kind: functional
origin: stressor(the-mint-runs-with-no-network)
satisfied-by: [Seats.Mint]
deferred:
---

# the refresh never blocks the mint

The mint refreshes the base it is about to branch from — the home
branch and every cascaded member's base — with a fetch of that branch
alone. The attempt is best-effort: no remote, no network, no upstream
or a refusal leaves the mint working from the local ref, and the report
names which ref it used. `--no-fetch` skips the attempt outright. The
refresh writes remote-tracking refs and nothing else, so no working
tree moves.

## System Context

Archi reaches the network in one place today, the push inside the
landing. The mint is the second, and for the same reason: a base an
operator cannot see is stale silently, while a failed fetch is loud and
costs one line. A pull would move a checked-out branch and could merge
inside someone's tree, so the refresh fetches and never pulls.

## Satisfy

`Seats.Mint` fetches the base branch before it resolves the branch
point, degrades to the local ref on any failure, and carries the
outcome into the report it already prints.

- test — a mint with an unreachable remote succeeds, branches from the
  local ref and says so; `--no-fetch` never contacts the remote; a
  member whose checkout is dirty or on another branch is refreshed with
  its working tree untouched
