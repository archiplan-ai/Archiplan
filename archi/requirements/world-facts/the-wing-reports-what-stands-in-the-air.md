---
kind: functional
origin: intent
satisfied-by: [DocsCompiler]
deferred:
---

# The wing reports what stands in the air

`check` reads the `uses` graph and the two other lists, and reports the decay of the
wing beside the findings it already gives. A cycle in `uses` is an error. Three
findings follow: `world_orphan` — nothing names this fact in `uses` and its `covers`
is empty, so nothing rests on it; `world_inferred` — `uses` is non-empty while
`sources` is empty, so the fact was derived and never observed; `world_chain_deep` —
the `uses` chain runs deeper than three, so the wing holds a theory of the world
instead of a record of it.

## System Context

The wing exists so that a changed world can drop the behavior it used to justify, and
that only works while the graph stays readable. Each finding names one way the
readability goes. An orphan is dead weight. A derived fact feels solid because it
follows from a checked one, yet nothing anchors it to the world, so it borrows a
credibility it does not have. A deep chain puts the blast radius of a deletion past
what a person can hold. None of the three blocks, because none of them is wrong —
they are the worklist for the next reading of the wing.

## Satisfy

`DocsCompiler` (walks the `uses` graph over the loaded world facts; a cycle is a
located error naming the ring; the three findings are emitted per fact with its path).

- test — a cycle of two facts in `uses` raises a located error naming both
- test — a fact nothing names, with an empty `covers`, reports `world_orphan`
- test — a fact with `uses` set and `sources` empty reports `world_inferred`
- test — a chain of four facts reports `world_chain_deep` and a chain of three does not
- test — the three findings never set a non-zero exit code
