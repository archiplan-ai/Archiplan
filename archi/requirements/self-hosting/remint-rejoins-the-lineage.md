---
kind: non-functional
origin: stressor(both-mint-the-next-id)
satisfied-by: [Archive]
deferred:
---

# Remint rejoins the lineage

When two branches mint the same version id, the losing save has a verb-shaped path back onto
the merged lineage: the post-merge collision state is detected and named with its recipe
instead of a parse-error cascade, the remint carries the discarded entry's note instead of
relying on memory, the reminted round's `closed:` stamp moves with it so the session record
stays true, and a save's artifacts — manifest entry, patch or keyframe, session stamps — are
checked as one travelling unit so a half-committed save is named at its author, not discovered
as a raw read error at every clone.

## System Context

Everything is files in git; the archive's invariants (dense ids, linear parents, sealed hashes)
span the manifest and the patch files, so a branch-parallel save collides in exactly two files
while the rest of both writers' work merges clean. Pressed for real by merge-pressure.

## Satisfy

`Archive.remint` mints the merged tree like a save and re-stamps the named session's `closed:`
to the new id, so the round record follows its answers onto the merged lineage. The archive's
diagnostics name their recipes: conflict markers in the manifest point at the remint path
instead of cascading through every session, and a manifest entry whose file is missing names
the half-shipped save. Every mint prints its artifacts as one commit unit.

- test — remint after a lineage collision mints and re-stamps the loser's session `closed:`
- test — conflict markers in the manifest yield one recipe-naming error, not a session cascade
- test — a manifest entry without its patch file names the half-shipped save at check
