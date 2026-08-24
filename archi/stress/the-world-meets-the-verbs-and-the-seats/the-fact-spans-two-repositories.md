---
affects: [Members, WorldDoc, Planner]
outcome: breaking
---

# The fact spans two repositories

A fact covers nodes realized in two member repositories. Its scenario walks from one to
the other, because that is what the person in the world actually does. The plan resolves
members one at a time, tasks declare outputs per member, and a scenario's steps have to
run somewhere — but a step in the backend and a step in the client have no single runner
and no single tree. Nothing in the world says where a cross-member scenario executes.

## Attractor

The scenario is written, parses, and is never anchored to code: no member owns it, so no
link is authored in either. It prints at the close of whichever plan touches either node,
gets latched, and stands as the most convincing unverified artifact in the tree —
executable in shape, unrunnable in fact.

## Resolution

A scenario names its runner with a tag — derived `a-scenario-names-where-it-runs` — and a
walk that crosses members is written as one scenario per member, held together by the
fact that carries them. The tag costs no new syntax, because
`the-grammar-takes-the-whole-language` already bought tags. Requiring one scenario to span
members was refused: multi-repo work resolves member by member everywhere else in this
tool, and a scenario with no home is the most convincing unverified artifact the world
could produce.
