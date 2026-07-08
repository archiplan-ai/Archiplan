---
affects: [Search]
outcome: breaking
---

# The plural misses the singular

An agent searches "folding sessions". The corpus says `fold`, `folded`, `session
fold`, `Sessions.fold` — and an exact-token matcher returns nothing for `folding`,
one weak hit for `sessions`. The agent concludes the KB is silent on folding and
re-derives from scratch what three stress rounds already settled.

## Attractor

Queries arrive in natural language; the corpus speaks in slugs, camel-case paths and
whatever inflection the author reached for. Exact tokens miss `versioning` against
`versions`, `remint` against `reminted`, `SourceTree` against "source tree". A full
stemmer is a heavy dependency with its own wrong answers; no normalization at all
makes retrieval a spelling exam the agent didn't know it was taking.

## Resolution

Broke exact-token matching: both sides normalize — lowercase, identifier splitting
(camel-case, dots, hyphens) so `Sessions.fold` yields `sessions` and `fold` — and a
shared-prefix rule scores at half weight: three characters or more of common prefix,
covering at least half of the longer token. The first cut of that rule (the shorter
token prefixing the longer whole) failed this stressor's own example — `versions` is
no prefix of `versioning`, they diverge at the eighth character — which is why the
floor is proportional, not a whole-token containment. Answered by
`matching-forgives-the-phrasing`.
