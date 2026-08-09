---
kind: functional
origin: stressor(the-local-branch-is-ahead-of-its-remote)
satisfied-by: [Seats.Mint]
deferred:
---

# the branch point follows the divergence

After the refresh the mint compares the local base with its remote
counterpart and branches from the safe side. Behind the remote takes
the remote ref. Ahead of it, or diverged from it, takes the local
branch, because unpushed commits are work. No upstream, or no
difference, takes the local ref. The report names the chosen ref, its
short commit and the divergence that decided it, so a seat carrying
unpushed commits says so at birth. An explicit `--base` overrides the
whole decision.

## System Context

Both directions of staleness cost real work: an old base meets the
world as conflicts, and a base that skips unpushed commits meets it as
missing code — the incident that opened this line of work. Naming the
divergence at the mint is what turns either one into a sentence the
operator reads instead of a surprise a week later.

## Satisfy

`Seats.Mint` counts both sides with a range read against the remote
ref, picks the branch point by that count, and prints it.

- test — a base behind its remote branches from the remote ref; a base
  ahead branches from the local one and the report names the count; a
  diverged base branches locally and names both counts; `--base` wins
  over all of it
