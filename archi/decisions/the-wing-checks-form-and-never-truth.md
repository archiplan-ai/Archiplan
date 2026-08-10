---
links: [world-facts, the-wing-reports-what-stands-in-the-air, a-world-fact-carries-its-scenarios]
prefer: [simplicity]
over: [correctness]
---

# The wing checks form and never truth

`check` decides the shape of a world fact, resolves its three lists and walks the
`uses` graph. It never decides whether the fact is still true. No command can: the
referent is outside the repository, and nothing in a tree of text observes the world.

We take that cost, and it is the largest one the wing carries. A false fact is worse
than a missing one, because it justifies a behavior with authority — the file stands,
the references resolve, the scenario runs green, and the condition under all of it
went away a year ago. The scenario lends its run to the fact beside it, so the wing
is built to keep the two apart: the scenario carries the state of its run, the fact
carries its sources and nothing more.

The alternative is machinery that watches the world — a review cadence, an external
feed, a staleness daemon. Each one is a second system to keep alive for an answer it
can only guess at. Instead the wing stays small enough to re-read whole, an empty
`sources` marks a fact nobody grounded, and the entry point is surprise: when a thing
happens that nobody expected, the first question is which recorded fact it
contradicts.
