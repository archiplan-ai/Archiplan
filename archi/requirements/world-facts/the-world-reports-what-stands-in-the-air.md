---
kind: functional
origin: intent
satisfied-by: [DocsCompiler]
deferred:
---

# The world reports what stands in the air

`check` walks the `uses` graph beside the two other lists and reports the decay of the
world. A cycle in `uses` is a located error naming the ring. `world_chain_deep` reports a
`uses` chain that runs deeper than three, because the world then holds a theory of the
world instead of a record of it, and the blast radius of a deletion stops fitting in a
head. Nothing here blocks except the cycle.

## System Context

The world exists so that a changed world can drop the behavior it used to justify, and
that only works while the graph stays readable. This claim owns the shape of the graph.
The state of a single fact is owned elsewhere, because three rounds showed those states
were scoped wrong when they lived here: `an-ungrounded-fact-says-so` reports a fact with
no sources, `a-fact-may-stand-before-the-model-does` separates a fact the model has not
reached from one that is dead weight, and
`the-fact-speaks-the-world-and-check-says-when-it-does-not` reports a fact that drifted
into the other world's vocabulary.

## Satisfy

`DocsCompiler` (walks the `uses` graph over the loaded world facts; a cycle is a located
error naming the ring; the depth finding is emitted per chain with its facts).

- test — a cycle of two facts in `uses` raises a located error naming both
- test — a cycle of one fact naming itself raises the same error
- test — a chain of four facts reports `world_chain_deep` and a chain of three does not
- test — the depth finding names every fact on the chain
- test — the depth finding never sets a non-zero exit code
