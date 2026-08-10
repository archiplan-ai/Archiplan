---
kind: functional
origin: intent
satisfied-by: [Planner, DocsCompiler]
deferred:
---

# Coverage reaches down the graph

A node is covered when a world fact names it in `covers`, or when it is reachable from
such a node along the declared connection edges, or when it is a child of a covered node.
`plan verify` names every task whose node is covered by none of the three, and says that
no recorded behavior reaches it. The report is advisory: an internal node that no
behavior reaches is a normal node, and the operator either writes the fact or reads the
line and moves on. `archi check` says nothing about node coverage on any tree.

## System Context

Scenarios run from the surface inward, so covering the few nodes a person touches covers
most of a tree through the edges already declared — and that is why a rule demanding a
fact per node was refused: it would fire on every lexer and canonizer in the model and
teach an operator to mute the class on the first day, which
`the-wing-arrives-without-noise` settled.

What this claim adds is the timing. The plan is where the wing is at risk of being routed
around: an author holding a task, seeing no scenario for its node, writes one into the plan
by hand — which is exactly the habit the wing replaced. Two claims already guard the empty
case, and neither guards this one: a plan can carry facts, close on a full block, and still
hold a task nobody has said anything true about. The line arrives while the work is being
planned, which is the last moment writing a fact is cheaper than not having one.

## Satisfy

`DocsCompiler` (computes the covered set from the facts' `covers` by forward reachability
over connection edges and by containment, against the plan's pinned version). `Planner`
(`plan verify` names each task whose node falls outside that set).

- test — a task over a node a fact names directly is not reported
- test — a task over a node reachable from a covered node is not reported
- test — a task over a child of a covered node is not reported
- test — a task over a node nothing reaches is reported by name
- test — the report never changes the exit code and never blocks a wave
- test — `archi check` reports nothing about node coverage on the same tree
