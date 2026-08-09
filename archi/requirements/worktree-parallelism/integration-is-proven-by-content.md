---
kind: functional
origin: stressor(a-squashed-pull-request-breaks-the-ancestry-proof)
satisfied-by: [Seats.Landing]
deferred:
---

# integration is proven by content

A landed side counts as integrated when the receiving branch carries
its work: the landed head is an ancestor of the receiving branch, or
the receiving branch holds the same content on every path the landing
touched. The comparison is one-directional and path-scoped — the paths
come from the landing against its merge base — so work the receiving
branch gained meanwhile never hides the proof. Ancestry is the cheap
first pass; content answers the squash, where the forge rewrites every
sha. Both tests read only — no merge is ever run to find out.

## System Context

Squash is the default merge button on GitHub, so an ancestry-only proof
answers "not integrated" forever on most teams. A whole-tree comparison
fails just as badly: a receiving branch that moved on for any other
reason answers "not yet" for every seat. Scoping to the landing's own
paths leaves one honest error — the receiving branch rewrote those very
files differently — and that error keeps a seat standing, the safe
direction.

## Satisfy

`Seats.Landing` probes with `merge-base --is-ancestor` and, failing
that, compares the receiving branch against the landing sha over the
paths the landing touched — the paths taken from its merge base, the
comparison read-only and quiet-or-differs.

- test — a fast-forward merge proves integrated by ancestry
- test — a squashed equivalent commit proves integrated by content
- test — a receiving branch carrying unrelated work of its own still
  proves the landing integrated
- test — an unmerged branch proves neither, and the seat stands
