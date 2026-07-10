---
kind: functional
origin: stressor(the-checkout-that-stayed-home)
satisfied-by: [Links]
deferred:
---

# Absence is not drift

A link into a member with no local checkout grades Unreachable — a state of its own, upstream of
Missing: no confidence observation is emitted, nothing decays, `--prune` never retires what a
scan merely could not see, and the report counts unreachable links per member. The one place
absence fails hard is an explicit ask: `verify --repo <member>` treats that member's absence as
the error it is.

## System Context

The half-checkout is the normal state of a multi-repo team, not an edge case; decay events are
journaled, so a wrong observation replays forever. Distinguishing "resolves to nothing" from
"nowhere to resolve" is what keeps automatic hygiene from corroding the record.

## Satisfy

`Links` (`Grader` grades Unreachable before Missing; observation events are emitted only where a
tree was actually read; `--repo` scope turns absence into failure).

- test — verify with an unmapped member grades its links Unreachable and emits no decay events
- test — `audit --prune` leaves Unreachable links untouched
- test — `verify --repo <member>` exits nonzero when that member is unreachable
