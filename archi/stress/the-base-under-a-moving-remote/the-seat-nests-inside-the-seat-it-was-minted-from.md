---
affects: [Seats.Mint]
outcome: breaking
---

# the seat nests inside the seat it was minted from

Follow the briefing: work that builds on an unlanded unit is minted from
inside that unit's worktree, so the fork grows from its branch. Run the
mint there.

## Attractor

The new seat lands in a folder beside its parent seat, one level deeper
every time — `…-worktrees/<a>-worktrees/<b>`. Paths grow, the registry
records them, and a cleanup of the parent takes its child with it.

## Resolution

A seat folder always sits beside the repository's main checkout, wherever
the mint runs from — the anchor member worktrees already use. Derived
`a-seat-sits-beside-the-main-checkout`.
