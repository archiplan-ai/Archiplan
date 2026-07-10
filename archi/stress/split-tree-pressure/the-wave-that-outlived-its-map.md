---
affects: [Links.Capture, Members]
outcome: surviving
---

# The wave that outlived its map

A wave opens on one machine and closes on another where the checkouts sit elsewhere; mid-wave, a
new member is declared and mapped. The index on disk was taken under a mapping that no longer
exists.

## Attractor

Capture diffs the new machine's trees against the old machine's paths and reads relocation as a
total rewrite — every symbol touched, every ref pressed, confidence splattered across the plan.

## Resolution

Survives by identity keys: the index is keyed by member name, never by resolved path, so a moved
checkout re-resolves and diffs clean. The member declared after open is outside the recorded
scan set — capture skips it with a note naming the recovery instead of diffing it against an
index that never saw it.
