---
kind: functional
origin: stressor(the-fact-dies-and-the-link-survives)
satisfied-by: [DocMint, Links]
deferred:
---

# Removal names the code it strands

`archi world rm` pre-flights the link journal beside the `uses` graph. When a link
anchors a scenario of the fact being retired, the command refuses and lists those
links by id and by the code item each one names. The operator retires them with the
existing `link rm --spec`, then runs the removal again.

## System Context

Deletion is the purpose of the wing, so it runs often, and every run that leaves a
link pointing at a scenario that no longer exists teaches the operator to skim
`link verify`. A grade that cries over correct code is worse than no grade. The
pre-flight is the same move the wing already makes for `uses` — name the blast
radius before the file goes — extended to the second thing that can depend on a
fact. It refuses rather than cascades: a link is a recorded human assertion, and no
verb retires one on the operator's behalf.

## Satisfy

`DocMint` (the removal reads the folded live link set beside the `uses` inverse, and
refuses with both lists when either is non-empty). `Links` (serves the folded set to
the pre-flight, as it already serves the plan's coverage gate).

- test — `world rm` on a fact whose scenario a link names refuses and lists the ids
- test — the refusal names the code item of each stranded link, not the id alone
- test — after `link rm --spec` retires them, the same `world rm` proceeds
- test — a fact with no links and no dependants retires in one call
- test — the refusal lists `uses` dependants and stranded links together in one message
