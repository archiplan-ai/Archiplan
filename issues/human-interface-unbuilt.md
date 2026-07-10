# The human interface exists only as an aspiration

**Kind:** missing feature (product surface) · recorded at the legacy-requirements migration (2026-07-10)

The legacy spec kept a wishlist for a human-facing surface that was never designed or
pressed through the loop:

- requirements viewed as a graph; stress sessions as tables (stressor, attractor,
  outcome, derived requirement slugs); the model as a versioned graph;
- live visibility of agent progress — new sessions, stressors, spec evolution — at a
  high level, without the tool-call noise;
- operator-facing vocabulary translated away from the theory (NKP, epistatic/epistemic,
  corridors, attractors) into terms a working developer already holds;
- a way to *address* the system when reporting a problem: runtime logs and rendered UI
  carrying node and requirement ids, so "this is broken" can name the element instead of
  paraphrasing the spec. This is the sharpest recorded pain — a visual bug today has no
  addressable way in, and agents fix the wrong thing.

## Impact

Archi is fully drivable by agents and CLI-literate humans; everyone else has no read
surface at all, and bug reports arrive unanchored.

## Fix shape

When any slice of this becomes real work, it enters as an intent through the ordinary
loop (problem statement first, requirements derived, model pressed). The id-addressability
item is probably the first candidate: it is small, testable, and unblocks the rest.
