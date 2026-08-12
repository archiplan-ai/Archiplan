---
kind: functional
origin: stressor(the-world-is-known-before-the-model)
satisfied-by: [WorldDoc, DocsCompiler]
deferred:
---

# A fact may stand before the model does

`covers` may be empty. An empty `covers` is a legal state, not an error and not an
orphan: it says the condition is recorded and the model has not reached it yet.
`world_uncovered` reports it so the gap stays visible, and `world_orphan` narrows to the
fact that nothing names in `uses` and that carries no scenarios either — dead weight
rather than early work. A missing or empty `Scenarios` block is itself a blocking error, so
that state is reachable only beside one: `world_orphan` marks a file that is broken and
unreferenced, and never a healthy fact.

## System Context

The chain this world serves runs from the world into the model, so on a new project the
conditions are known first and there is nothing to cover. A tool that demands elements
before the first fact inverts its own design: the author builds the model from intuition
and back-fills facts over what they already made, which is a justification layer, not a
condition. Requirements already work the right way here — `satisfied-by` stays open and
`unsatisfied_requirement` is a finding and not an error — and the world joins that pattern
instead of inventing a stricter one.

## Satisfy

`WorldDoc` (an empty `covers` is a state the file may wear). `DocsCompiler`
(`world_uncovered` for an empty `covers`; `world_orphan` narrowed to no inbound `uses`
and no scenarios).

- test — a fact with empty `covers` passes `check` and reports `world_uncovered`
- test — a fact with empty `covers` and one scenario is not an orphan
- test — a fact with no inbound `uses` and no scenarios reports `world_orphan`
- test — filling `covers` later clears `world_uncovered` with no other edit
- test — neither finding changes the exit code
