---
affects: [Seats.Registry, Seats.Guard]
outcome: breaking
---

# a closed row still licenses mutation

Close a seat, then run a mutating command from a checkout whose path a
closed row still names — a folder recreated by hand at the same place,
or a member checkout the landing left standing.

## Attractor

The registry answers "bound" from a row that describes finished work.
The guard opens, mutations land in a workspace nobody works in, and the
record becomes a set of stale licenses.

## Resolution

Only an active row licenses. Every lookup that grants — the mutation
guard, the verdict gate, the plan owner, the member resolution — reads
active rows alone, and a closed row is history that answers nothing.
Derived `only-an-active-row-binds`.
