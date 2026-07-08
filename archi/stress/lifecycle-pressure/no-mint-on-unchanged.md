---
affects: [Archive]
outcome: surviving
---

# No mint on unchanged

The fix presses the archive's core rule from the other side: if closing a round wants a version
id, the cheapest patch is to mint one — an empty patch, a token model edit, a `closed-round`
pseudo-version.

## Attractor

Cheap versions drown the anchors: stress sessions, plans and links all pin version ids, and an
archive where ids mint for lifecycle convenience stops meaning "the model changed here". The
dense sequence and the patch files degrade into ceremony, and `versions-mint-on-meaning` dies by
a thousand conveniences.

## Resolution

Holds on v0004 and fences the fix: `Archive.save` still mints nothing when the render hashes
equal to the latest entry — the close borrows the *current* id instead of minting a fresh one,
and two rounds may legitimately close against the same version. `versions-mint-on-meaning`
keeps its claim; a regression asserts the archive still holds exactly one entry after a
close-without-mint.
