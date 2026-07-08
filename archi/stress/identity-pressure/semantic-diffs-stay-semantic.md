---
affects: [Renderer, Archive]
outcome: surviving
---

# Semantic diffs stay semantic

The other half of the iff: a real edit — an edge added, retyped, re-endpointed — must still move
the hash, and the patch between two renders must still read as the semantic delta it is.

## Attractor

The sort overreaches: deduplication or normalization sneaks in with the ordering and two
distinct models render identically, or the sort scatters an inserted statement's context so a
one-edge change diffs as a wall of moved lines — the patch files stop being readable records and
versions stop being trustworthy identities.

## Resolution

Holds on v0004 and fences the fix: sorting is a permutation, never a projection — every
statement keeps exactly one line, so distinct models keep distinct renders. And a total order
*improves* the diff property authoring order only approximated: an inserted edge lands at its
sorted position as one `+` line, shifting nothing — where under authoring order a module rename
could shift eighteen. Pinned by the strengthened renaming-invariance regression asserting
bit-identical batches across renames, beside the existing semantic-change tests that assert the
hash moves.
