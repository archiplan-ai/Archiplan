---
affects: [Archive, Cli]
outcome: surviving
---

# Changed rounds still mint

The ordinary round: breaking stressors answered with model edits, the tree's render no longer
hashes to the pinned version, `archi version save` runs at the close.

## Attractor

The no-op path overreaches: "save closes the round" hardens into closing *before* minting, or
the unchanged branch swallows the changed one — and a real round either closes against the stale
id or stops minting altogether, splitting the round's record from its version.

## Resolution

Holds on v0004 and fences the fix: the `Written` path is untouched — a changed model mints its
version and the session closes against the *minted* id, exactly as every prior minting round
closed. Pinned by a regression: model edit + open session, one save → v0002 minted, session
stamped `closed: v0002`.
