---
affects: [SourceTree, Compiler, CanonicalRender, RequirementDoc]
outcome: breaking
---

# Clean merge, broken contract

Edits that never touch the same file: one branch deletes a node, the other adds an edge
carrying it; one branch retypes an edge's carrier, the other writes a requirement claiming the
element it hangs off; one branch names a requirement and the other a session with the same
slug. Git merges every case without a murmur.

## Attractor

main compiles broken out of a conflict-free merge — or compiles green while meaning something
neither author reviewed.

## Resolution

Broke on placement, twice over. Where the semantic conflict is structural the tool is loud and
located — `E_UNKNOWN_NAME` at the orphaned edge, `E_SLUG` naming both colliding files — but
only after the merge, at the integrator's desk: neither author's branch could show it, so the
first check that can fail runs on a merge commit nobody authored. Where the conflict is not
structural it is silent: the retyped carrier slid under the fresh claim with everything green,
and nothing renders what the merge did to the model — `version current` says only "dirty",
`version diff` takes archived ids only, and the lowered batch and an archived render don't
compare. The first surface where the merge's semantic delta becomes visible is the next
`version save`: after the mint, conflated with the round's own answers, reviewable only as
history. The canonical render is the contract, but the merge is reviewed as text.
Derived: merge-deltas-are-reviewable.
