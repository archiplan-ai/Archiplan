---
kind: functional
origin: stressor(a-squashed-pull-request-breaks-the-ancestry-proof)
satisfied-by: [Seats.Landing]
deferred:
---

# integration is proven by content

A landed side counts as integrated when the receiving branch carries
its content: the landed head is an ancestor of the receiving branch, or
the diff between them is empty. Ancestry is the cheap first pass;
content answers the squash, where the forge rewrites every sha. Both
tests read only — no merge is ever run to find out.

## System Context

Squash is the default merge button on GitHub, so an ancestry-only proof
answers "not integrated" forever on most teams. A content proof can
under-report when the receiving branch moved on and rewrote the same
files, and that error keeps a seat standing — the safe direction.

## Satisfy

`Seats.Landing` probes with `merge-base --is-ancestor` and, failing
that, an empty tree diff against the receiving branch.

- test — a fast-forward merge proves integrated by ancestry
- test — a squashed equivalent commit proves integrated by content
- test — an unmerged branch proves neither, and the seat stands
