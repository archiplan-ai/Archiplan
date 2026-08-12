---
affects: [WorldDoc, RequirementDoc, DocsCompiler]
outcome: accepted
---

# Both sides assert the same behavior

Write a requirement whose `- test —` bullet asserts what a world fact's scenario
already asserts, over the same node. Origin is what divides the sides, so both
records are legal and both resolve. Nothing compares them. Change the behavior and
one of the two turns false while `check` walks every reference, finds each one sound,
and reports nothing at all.

## Attractor

One behavior, two sources of truth, drifting. The reader trusts whichever file they
opened first, and the two records — meant to hold a node from opposite sides — start
holding it from the same side with different words.

## Resolution

Accepted. No cheap detector exists: a prose verification bullet and a Gherkin step
cannot be compared by a machine, and the pair of records that would be compared is
legal by construction. Two things hold the overlap down instead, and neither is a
check — the origin rule, which sends a claim to one side or the other before it is
written, and the size of the world, which `the-check-counts-the-world` now reports. The
sacrifice is signed in `one-behavior-can-be-asserted-twice`.
