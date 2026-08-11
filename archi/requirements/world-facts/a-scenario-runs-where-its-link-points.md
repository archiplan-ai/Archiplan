---
kind: functional
origin: stressor(the-fact-spans-two-repositories)
satisfied-by: [Links, WorldDoc]
deferred:
---

# A scenario runs where its link points

A scenario carries no declaration of where it runs. The link that anchors it already says
so: an anchor reads `[<member>//]<file>[#<symbol>]`, so a scenario anchored to
`backend//tests/walk.rs#opens` runs in `backend`, and one anchored to a bare path runs in
the project's own repository. A fact whose scenarios run in two members writes one
scenario per member and anchors each where it runs.

## System Context

The pressure was real: a fact can cover nodes in two repositories, and a single walk
across them has no one place to execute. The first answer was a tag on the scenario naming
its member. That tag was written zero times and it was a second copy of something the tool
already held — worse, a copy nothing reconciled, so a scenario could claim one member and
anchor into another with no complaint.

Where a scenario runs is not a property of the scenario. It is a property of the code that
answers it, and the anchor is where that code is named. Nothing new is built here: this
claim records that the question was already answered, so the next reader does not invent a
second mechanism for it.

## Satisfy

`Links` (the anchor's member prefix is what says where a scenario runs; no separate
declaration is parsed or stored). `WorldDoc` (a scenario carries its name and its steps and
nothing about repositories).

- test — a scenario anchored to `<member>//<file>#<symbol>` reports that member as its runner
- test — a scenario anchored to a bare path reports the project's own repository
- test — a fact holding two scenarios anchored into two members passes `check`
- test — a scenario body naming a member raises no special handling: it is prose in a step
