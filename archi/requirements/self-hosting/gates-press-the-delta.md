---
kind: functional
origin: stressor(gate-demands-untouched-surface)
satisfied-by: [Planner]
deferred:
---

# Gates press the delta

The wave-close coverage gate demands asserted links only for the spec_refs the wave's delta
presses — refs some claimed changed item of the task carries signal for, by the same term test
capture mints with. Unpressed refs never block; when uncovered they surface as a suggested
checklist of exact `archi link add <ref> <file#symbol> --kind indirect` lines, printed both when
the gate blocks and when the wave closes, so hand-authoring the untouched surface is a named,
voluntary move instead of a coerced one. An asserted link satisfies its ref however it was born,
and a delta pressing nothing closes its wave without demanding links.

## System Context

Replayed from `issues/wave-gate-covers-the-node-not-the-delta.md`: a one-verb fix on a hub node
left the gate listing 10 uncovered refs whose only in-delta candidates were false claims —
`Cli.check ← main.rs#run_version` one confirm away from asserting what the code does not do.
The first wave on any hub node paid a link-authoring tax unrelated to its change, under gate
pressure — exactly when rushed links get minted — and the expected hand-authoring move was
discoverable only by usage error.

## Satisfy

`Planner` (capture returns the pressed refs per task with its outcome; `Planner.coverage` gates
on the pressed subset and renders the unpressed remainder as the checklist through
`Planner.advance`).

- test — plans: a one-port fix on a hub node closes its wave with asserted coverage of the pressed refs only
- test — plans: the blocked message and the wave-close output name the unpressed uncovered refs as link add suggestions
- test — plans: a delta with no signal for any ref closes its wave without demanding links, and an asserted link on an unpressed ref silences its suggestion
