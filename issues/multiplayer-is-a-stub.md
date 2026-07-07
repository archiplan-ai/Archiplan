# Multiplayer is a one-line stub — and the first stress round hit it

**Kind:** missing feature (spec stub) · surfaced as a breaking stressor in the bootstrap

`requirements/multiplayer.md` is a bare title. The very first stress round over the self-model
pressed it for real: the breaking stressor
`archi/stress/first-pressure/parallel-rounds.md` — two agents on two branches each open a
session against the same version and save — leaves the merged tree with two open sessions
(`E_SESSION`) and colliding `v0002` ids (dense-sequence check + ordinary git conflicts).

## Impact

The collisions are loud by design, but loud is not answered: nothing helps the second writer
re-mint a save onto the merged lineage, fold two concurrent rounds into one record, or hold a
session-open discipline that spans branches rather than working trees. The repair is improvised
exactly when the operator is least equipped. The gap is recorded in the spec as the deferred
requirement `archi/requirements/self-hosting/parallel-editing-discipline.md` (origin:
`stressor(parallel-rounds)`), so it stays visible on every check until lifted.

## Fix shape

Write `multiplayer.md` for real. Minimum viable discipline: a `version remint` verb that
re-sequences an out-of-lineage save onto the merged manifest; a documented merge recipe for two
open sessions (pick one, fold stressors, re-pin); and a check hint that names the recipe when it
detects the post-merge state.
