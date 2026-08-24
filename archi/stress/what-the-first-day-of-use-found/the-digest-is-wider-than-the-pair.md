---
affects: [Links, Links.Canonizer, WorldDoc]
outcome: breaking
---

# The digest is wider than the pair

A link to a scenario records a digest of the fact it lives in, not of the scenario
itself. `world_check::scenario_digest` fingerprints the whole `Scenarios` block — the
feature line, every scenario name, every step. So a link on `fact#A` fails the moment a
step of `fact#B` is reworded in the same file, over code nobody touched. The agent that
wrote it reported this in the same breath as the work: the shared digest was the price of
one function, and one function was the point.

## Attractor

The operator learns that a failing scenario link means "something in that file moved",
which is not what the record claims. Repinning becomes a chore performed without reading,
and the pair stops witnessing anything — the exact outcome
`a-scenario-link-is-a-witnessed-pair` weighed and accepted a narrower cost to avoid. A
fact holding four scenarios makes every one of them decay together, so the more a
condition is elaborated the less its links are worth.

## Resolution

The grain becomes an argument on the one shared function: derived
`the-digest-witnesses-one-scenario`. The link fingerprints the scenario it addresses; the
plan's drift line still asks for the whole block by naming none. One contract, two grains,
no second digest — which is what the fold was for.
