---
kind: functional
origin: stressor(the-block-walks-outside-the-plan)
satisfied-by: [Planner, WorldDoc]
deferred:
---

# The block marks what lies outside the plan

A fact joins the closing block when it covers any node the plan holds a task for, and
the block says which of that fact's covered nodes this plan never built. A scenario
whose fact covers nothing outside the plan prints clean. The mark is on the fact, not on
the individual step, because `covers` is what the plan can resolve and a step's reach is
prose.

## System Context

Facts cross plans by design — a condition on the world does not stop at the edge of a
piece of work — so a closing block that only ever held wholly-contained facts would be
empty on most plans, and the ceremony would be worth nothing. The other failure is worse
though: a block that silently asks for verification of a path the plan had no parts for
gets latched anyway, and after two of those the latch is a keystroke. Naming the gap
keeps the ceremony honest in both directions: the operator sees what is verifiable here
and what waits on another plan.

## Satisfy

`Planner` (resolves each collected fact's `covers` against the plan's task nodes and
prints the uncovered remainder beside the scenario). `WorldDoc` (`covers` as the list
the mark is computed from).

- test — a fact covering four nodes on a one-node plan prints the other three as outside
- test — a fact whose covered nodes the plan all holds prints with no mark
- test — the mark names node paths, not step text
- test — the mark changes no latch and no exit code
