---
affects: [WorldDoc, PlanFile, Planner]
outcome: breaking
---

# Two things are called a scenario

Read `scenarios-close-the-plan` beside `a-world-fact-carries-its-scenarios`. The plan's
scenarios are free-text user stories on the plan envelope, and that requirement states
their decoupling as a reason: a story crosses many requirements across many nodes, so
pinning it to one element would lie about its scope. The world fact's scenarios are
Gherkin, and `covers` pins them to nodes. One word names both. Their shapes differ,
their lifetimes differ — a plan's stories go when the plan completes, a fact's stay —
and the standing requirement argues against exactly what the new wing does.

## Attractor

An operator asks `archi plan scenarios list` and gets one of the two kinds without
being told which. A reader who learned the word in the plan carries the wrong model
into the wing, and the reverse. The wing looks like a durable home for the plan's
stories, people start writing them there, and the plan's exit ceremony quietly loses
the block it latches on.

## Resolution

One word, one thing: the wing owns scenarios and the plan authors none. Derived
`the-plan-closes-on-the-world-s-scenarios`, and `scenarios-close-the-plan` is rewritten
to own the ceremony and the latches alone. The reason the plan's stories were decoupled
from the spec survives the move — `covers` is a list, so a scenario that walks four
nodes names four and nothing is pinned to one element. The price is signed in
`the-stories-move-out-of-the-plan`.
