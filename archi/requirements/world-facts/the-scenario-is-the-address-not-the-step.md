---
kind: functional
origin: stressor(the-step-is-reworded-and-the-link-holds)
satisfied-by: [WorldDoc, Links]
deferred:
---

# The scenario is the address, not the step

A link into the wing addresses a scenario: `<fact-slug>#<scenario name>`. Steps are not
addressable. Rewording a step changes nothing about the link, because the step text was
never the reference; renaming a scenario breaks it, and that break is a located error the
operator repairs with `link repin`.

## System Context

Step text is English that a person rewrites for taste, and grading a link against it gives
two bad outcomes and no good one: either every rewording decays a link until decay means
nothing, or nothing decays and a step can be rewritten into a different assertion behind a
green link. A scenario name is a name — the same kind of thing as a slug or a symbol,
edited deliberately and rarely — so it grades like every other reference in this
repository. What the link buys stays intact: the anchored artifact is executable, so the
edge from spec to code is still decided by a run rather than by a reading.

## Satisfy

`WorldDoc` (the scenario name is the addressable unit within a fact; two scenarios in one
fact cannot share a name). `Links` (resolves `<fact-slug>#<scenario name>` as a spec ref,
grades it as it grades any other, and reports a renamed scenario as an unresolved ref).

- test — a link to `<fact-slug>#<scenario name>` resolves and verifies
- test — rewording a step under that scenario leaves the link untouched
- test — renaming the scenario reports the link as unresolved
- test — `link repin` moves the link onto the new scenario name
- test — two scenarios with the same name inside one fact raise a located error
- test — a link naming a scenario that no fact holds refuses at `link add`
