---
kind: functional
origin: stressor(two-things-are-called-a-scenario)
satisfied-by: [Planner, WorldDoc]
deferred:
---

# The plan closes on the world's scenarios

The plan keeps no scenario block of its own. After the last wave closes, `plan next`
prints the scenarios of every world fact that covers a node the plan holds a task for,
and latches on that block exactly as it latched on the free-text one. A plan whose
nodes no fact covers has no block and closes directly, as it does today. `archi plan
scenarios list` reads the same set.

## System Context

`scenarios-close-the-plan` set the stories free of the spec on a reason that was sound
for free text: a story crosses many requirements across many nodes, so pinning it to
one element would lie about its scope. A world fact does not pin a scenario to one
element — `covers` is a list, and a fact that conditions four nodes names four. The
scope survives, and what the plan gains is a block that was written once, kept beyond
the plan, and parsed by a grammar. The stories stop being authored per plan and start
being collected from the wing.

## Satisfy

`Planner` (at close, collects the world facts covering the nodes its tasks name, prints
their scenarios as the final step and drives the existing latch pair over that block).
`WorldDoc` (the durable home of the block: written once with its fact, read by every
plan that touches a covered node).

- test — a plan whose tasks touch a covered node prints that fact's scenarios at close
- test — the same fact covering two nodes prints once, not twice
- test — a plan whose nodes no fact covers refuses the final latch
- test — the latch pair runs over the collected block as it ran over the free-text one
- test — `plan scenarios list` and the close step read the same set
