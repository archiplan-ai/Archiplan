# `archi search` is fully spec'd and entirely absent

**Kind:** missing feature (spec'd) · confirmed during the self-hosting bootstrap
**Status:** resolved 2026-07-08 — `archi search` shipped (intent `agent-retrieval`, round `retrieval-pressure` @ v0010, plan `find-by-phrase`, commit cc85d07)

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

## Resolution

Shipped along the fix shape's first cut: `archi search <phrase>... [--kind k]... [--limit n]
[--json]` — ranked lexical scoring over cards (per-query document-frequency damping, shared-prefix
matching, name/summary/body field weights) across model elements with their identity prose,
intents, requirements, stressors and sessions; every hit carries its kind's next-hop refs, and
working-copy inclusion holds by construction (scan-on-query, no persisted index —
`search-reads-the-tree-it-stands-on`). The verb degrades per corpus instead of dying with the
model (`a-dark-corpus-stays-partial`). The `version checkout` reference was struck from
`search.md`: version-horizon search is recorded as a deliberate deferral
(`versions-stay-searchable`, deferred — the archive seals the model alone; a horizon that pins
elements beside live prose answers with a chimera). `crates/archi/src/search.rs` + `run_search`
in `main.rs`; 7 unit + 3 e2e tests; 41ms wall on this repository's own KB.
