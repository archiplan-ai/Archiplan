---
affects: [Search, SourceTree]
outcome: breaking
---

# A stale index lies

An agent edits `gates-press-the-delta.md`, renames the claim it makes, and immediately
searches for the new wording to cross-check its own edit. A prebuilt index still holds
the old tokens: the search returns the stale card, the agent concludes its edit never
landed, and re-applies it — or worse, trusts the index's snippet over the file.

## Attractor

Every write in this system is a bare text edit into the tree — no daemon watches, no
save hook fires, and half the writers are humans in editors archi never sees. Any
persisted index is a second copy of the truth with no invalidation signal, and a copy
that drifts silently is worse than no copy: it answers with confidence. The KB's own
requirement `source-is-the-only-truth` names the trap; an index file under `archi/`
would be the first store that requirement does not govern.

## Resolution

Broke the index before it was built: retrieval may keep no persisted derivative of the
corpus — every query scans the live tree and the freshly compiled model, so a mutation
is searchable in the very next call and `version save` is nowhere in the loop.
Answered by `search-reads-the-tree-it-stands-on`.
