---
kind: functional
origin: stressor(the-common-word-floods-the-ranking, the-plural-misses-the-singular)
satisfied-by: [Search]
deferred:
---

# Matching forgives the phrasing

Scoring meets the query halfway on both axes the round broke. Vocabulary: both sides
normalize — lowercase, identifier splitting on case, dots and hyphens — and a shared
prefix of three or more characters covering at least half of the longer token matches
at half weight, so `folding` reaches `fold` and `versioning` reaches `versions` with
no stemmer. Weighting: a term's
contribution is damped by its frequency across the corpus scanned in the same pass —
a token in every card contributes nothing, a rare token decides the ranking — and a
hit in an object's name or path outweighs the same hit in its body. The final order
is total and deterministic: score, then kind, then slug.

## System Context

Queries arrive in natural language; the corpus speaks in slugs, camel-case paths and
inflected prose, and its vocabulary is Zipfian — the domain nouns saturate it. Because
`search-reads-the-tree-it-stands-on` makes every query a full scan, corpus statistics
are computed fresh per query for free: no persisted vocabulary, nothing to go stale.

## Satisfy

`Search` (one tokenizer for query and corpus with case, dot, hyphen and camel-case
splitting; per-field weights with name over summary over body; per-query document
frequencies damp common terms; shared-prefix matches score half; ranking breaks ties
by kind then slug).

- test — `folding` finds a card whose text says `fold`, and `versioning` finds `versions`, both through the prefix rule
- test — a token present in every card contributes nothing: the ranking follows the rarer companion term
- test — a name hit outranks the same term appearing only in a body
- test — same tree, same phrase, twice: byte-identical output
