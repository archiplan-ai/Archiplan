---
affects: [Search, Archive]
outcome: breaking
---

# The archive answers yesterday

An agent debugging a pinned plan wants retrieval at the pin: `archi search --at v0008
"definitions"`. The archive reconstructs v0008's model faithfully — but the
requirements, stressors and sessions on disk are today's. The natural implementation
searches pinned elements beside live docs and returns one merged list: a chimera
wearing a version label, half yesterday, half now, indistinguishable per card.

## Attractor

`--at <id>` already works on `read` and `query`, so symmetry begs search to take it.
But the archive seals the canonical render — the model alone. The doc tree has no
archived form: its history lives in git, outside archi's seals. A version-scoped
search over the full corpus is unbuildable from what the archive holds, and the
almost-right version — pinned model, live prose, one label — misleads more than it
serves: yesterday's element card would carry today's requirement stamps.

## Resolution

Broke the symmetry: search takes no `--at` until the whole corpus can honor it.
The verb searches the live tree and the live model, full stop; version-horizon
retrieval is deferred, recorded with this reason, until doc sources version alongside
the render. Answered by `versions-stay-searchable` (deferred).
