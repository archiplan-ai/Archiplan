---
affects: [DocMint, Planner]
outcome: surviving
---

# Deferred mints park forever

`--deferred` is the one optional flag; a requirement minted deferred has empty
`satisfied-by`, never matches a task, never gates a plan — a parking lot with no
tow truck.

## Attractor

Deferred becomes the polite graveyard: claims recorded to be forgotten, check
green the whole time.

## Resolution

Held — deferral is a signed state, not an omission: the flag demands its reason,
`check` reports every deferred requirement with that reason on each run, and the
worklist keeps the row until someone either lifts the deferral or deletes the
claim through the pre-flighted `req rm`. Forgetting it now takes effort.
