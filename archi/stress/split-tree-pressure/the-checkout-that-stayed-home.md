---
affects: [Links.Grader, Members]
outcome: breaking
---

# The checkout that stayed home

A teammate clones only the spec repository and one of three members, then runs `link verify` and
`link audit --prune` out of habit. Two members' worth of links resolve nothing — not because the
code is gone, but because it was never there.

## Attractor

Absence reads as loss. Every link into an unmapped member grades Missing, evidence confidence
decays on each run, `--prune` retires links whose code is perfectly healthy two directories away
on someone else's machine — and the decay is journaled, so the damage replays forever. The
half-checkout, the *normal* state of a multi-repo team, becomes the state that corrodes the
record.

## Resolution

Unreachable becomes a grade of its own, upstream of Missing: no observation, no decay, no prune,
reported per member. Scoping a verify to a member you asked for (`--repo`) is the one place
absence fails hard. Derived: `absence-is-not-drift`.
