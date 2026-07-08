---
kind: non-functional
origin: stressor(clean-merge-broken-contract)
satisfied-by: [Archive]
deferred:
---

# Merge deltas are reviewable

The live tree's canonical render is diffable against any archived version before anything is
minted: the semantic delta of a merge — or of any dirty state — is reviewable, and CI-able, at
the integrator's desk, not first visible as the next version's patch after the save has sealed
it together with everything else in the round.

## System Context

The canonical render is the contract, but merges are reviewed as text: textually disjoint edits
compose into models that break loudly only post-merge, or drift silently under standing claims.
Pressed for real by merge-pressure.

## Satisfy

`version diff` accepts `live` on either side: the working tree compiles and renders canonical,
and diffs against any archived version — the merge's semantic delta is reviewable before a save
seals it, on any dirty tree, in CI.

- test — `version diff <id> live` on a dirty tree shows exactly the unsaved semantic delta
- test — `version diff live <id>` reverses the direction; two archived ids behave as before
