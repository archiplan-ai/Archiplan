---
affects: [Planner]
outcome: breaking
---

# Gate demands untouched surface

`plan task add` derives a hub node's whole spec surface, and the wave-close gate demands an
asserted link per derived ref — but capture only mints candidates from the wave's delta, so for
every ref the delta never touched the gate demands links capture cannot supply.

## Attractor

Replayed from `issues/wave-gate-covers-the-node-not-the-delta.md` as observed closing plan
`close-without-minting`: t1 on `Cli`, delta = `run_version`'s Unchanged arm, and the gate listed
10 uncovered refs. The only in-delta candidates for them were false claims — `Cli.check ←
main.rs#run_version` was one confirm away from asserting that `run_version` implements `check`.
Closing required discovering `link add --kind` by usage error and hand-authoring l0258–l0266,
the full dispatch table, under gate pressure — exactly when rushed links get minted.

## Resolution

Broke, as filed. Answered this round by scoping the demand to the delta: capture reports, per
task, the refs its claimed changed items carry signal for, and the coverage gate blocks only on
those. Refs the delta does not press surface as a suggested checklist — the exact
`archi link add <ref> <file#symbol> --kind indirect` lines, printed blocked or closing — naming
hand-authoring as the expected move instead of leaving it to usage-error archaeology. The
one-time hub-node traceability (the dispatch table) stays available, voluntary instead of
coerced. Derived: gates-press-the-delta.
