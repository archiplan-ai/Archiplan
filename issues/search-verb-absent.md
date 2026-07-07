# `archi search` is fully spec'd and entirely absent

**Kind:** missing feature (spec'd) · confirmed during the self-hosting bootstrap

`requirements/search.md` specifies natural-language retrieval over spec elements — ranked
results, self-contained element cards, working-copy inclusion — and the doc schemas are visibly
designed for it (summary-first bodies, heading-chunked sections, slugs as the reference
currency). No `search` verb exists in `main`'s dispatch; nothing implements any of it.
`search.md` also references a `version checkout` capability ("shifts the search horizon") that
`run_version` does not implement.

## Impact

The self-model already carries 17 requirements, 6 stressors and an intent; grep works at this
scale and will stop working quietly as the corpus grows. Retrieval is the read surface the
requirement format was shaped for — its absence means the format is paying costs (fixed summary
position, card-shaped sections) whose benefit hasn't shipped.

## Fix shape

Ship the spec'd verb (even a lexical BM25-over-cards first cut honors the interface), or mark the
spec deferred so the promise and the tool agree. Decide `version checkout` separately — either
implement it or strike the reference from `search.md`.
