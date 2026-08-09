---
kind: functional
origin: stressor(the-seat-nests-inside-the-seat-it-was-minted-from)
satisfied-by: [Seats.Mint]
deferred:
---

# a seat sits beside the main checkout

A seat folder is a sibling of the repository's main checkout, wherever
the mint runs from. Minting from inside another seat creates the new
folder beside the main checkout, never inside the parent seat.

## System Context

The briefing sends work that builds on an unlanded unit to be minted
from inside that unit's worktree, so the fork grows from its branch.
With the folder derived from the invoking checkout, that instruction
nested seats one level deeper each time. Member worktrees already
anchor on their repository's main checkout; the home side is the half
that was left behind.

## Satisfy

`Seats.Mint` derives the folder from the main checkout of the
repository, the same anchor the cascade uses.

- test — a mint run from inside a seat lands its folder beside the
  main checkout, and the registry records that path
