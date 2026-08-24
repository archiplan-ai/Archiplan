---
kind: functional
origin: stressor(the-checkout-that-stayed-home)
satisfied-by: [Links]
deferred:
---

# Absence is not drift

A link into a member with no local checkout grades Unreachable — a state of its own, upstream of
Missing: a tree nobody read is no observation, so nothing is written against those links and the
report counts them per member. The one place absence fails hard is an explicit ask:
`verify --repo <member>` treats that member's absence as the error it is.

## System Context

The half-checkout is the normal state of a multi-repo team, not an edge case, and the journal is
append-only, so a wrong observation replays forever. Distinguishing "resolves to nothing" from
"nowhere to resolve" is what keeps automatic hygiene from corroding the record.

## Satisfy

`Links` (`Grader` grades Unreachable before Missing; observations are written only where a tree
was actually read; `--repo` scope turns absence into failure).

- test — verify with an unmapped member grades its links Unreachable and writes nothing against them
- test — `verify --repo <member>` exits nonzero when that member is unreachable
