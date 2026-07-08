---
affects: [Search]
outcome: breaking
---

# The common word floods the ranking

An agent asks for "the version seal". In this corpus `version` appears in nearly every
session file, half the requirements and a dozen element definitions; `seal` appears in
five places, every one of them exactly what the agent wants. A raw match count ranks
the version-flooded cards first and the five sealing cards drown below the fold.

## Attractor

A KB's vocabulary is Zipfian: the words that name the system (`version`, `model`,
`link`) saturate the corpus precisely because the corpus is about them. Any scoring
that pays per occurrence hands the ranking to the words carrying the least signal, and
the phrase's discriminating token — the rare one — is outvoted by its common neighbor.
Stopword lists don't save it: this corpus's stopwords are domain nouns no fixed list
anticipates.

## Resolution

Broke flat scoring: a token's weight is set by the corpus being scanned at query time
— a term in every card contributes nothing, a term in five cards decides the ranking.
Scan-first-score-second makes the statistics free: the pass that reads every card
already knows every frequency, no persisted vocabulary needed. Answered by
`matching-forgives-the-phrasing`.
