---
affects: [Archive]
outcome: surviving
---

# Old versions stay reconstructable

The archive already holds five sealed versions rendered under the old contract: keyframes and
patches whose sha256 seals were minted over authoring-ordered bytes.

## Attractor

The contract change reaches backward: reconstruction re-renders instead of replaying stored
bytes, seals recompute against sorted output and every historical version reports corrupt — or
worse, the archive silently rewrites history to the new order and the seals become lies. The
ruler changed, and every past measurement gets blamed for it.

## Resolution

Holds on v0004 and fences the fix: reconstruction is byte replay — keyframes and patches apply
as stored, seals verify over stored bytes, and the contract that produced them is irrelevant to
their integrity. Old versions reconstruct exactly as sealed; they simply never hash-match a live
render again, which is correct — the live tree is *at* the first version minted under the new
contract, and the migration save records the reorder as one honest patch (`version diff
v0004 v0005` reads as pure permutation, the contract change made visible). The archive is
append-only through the change, like every other day.
