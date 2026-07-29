---
affects: [DocMint, Seats]
outcome: surviving
---

# Parallel seats mint one slug

Mint the same requirement title in two parallel seats: each `req add` checks slug
uniqueness against its own tree, both succeed, and the join lands two files
claiming one slug — or one path from both sides.

## Attractor

Write-time uniqueness quietly becomes merge-time breakage; agents trust the mint's
green answer and stop expecting slug conflicts at all.

## Resolution

Held — by the same contract every hand-written doc already lives under: tree-local
truth at the write, `E_SLUG` and content conflicts at the join, triaged by the
archi-merge ceremony. The verb adds no new failure surface; a cross-seat register
would add a shared mutable surface the seat model exists to avoid.
