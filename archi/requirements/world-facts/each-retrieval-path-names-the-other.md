---
kind: functional
origin: stressor(two-retrieval-paths-answer-differently)
satisfied-by: [Cli, Search]
deferred:
---

# Each retrieval path names the other

An empty result names the other path. `world ls --covers <node>` finding nothing says that
a phrase search may still reach the fact by its own words; `archi search --kind world`
finding nothing says that `world ls --covers` walks the bridge exactly. Neither verb
changes what it returns.

## System Context

The two verbs answer different questions and an operator has no way to know that from
either one. `covers` is exact and blind to wording; the phrase scan reads the fact's own
vocabulary, which by the content rule excludes the nouns of the model. So a search about a
node returns nothing about the fact conditioning it, and the operator concludes the wing is
empty rather than that they used the wrong door. `refusals-name-the-continuation` already
holds this shape for refusals; an empty answer is the same situation with a different exit
code.

## Satisfy

`Cli` (the empty-result line on the `world ls` and `search` renderings, human form and the
JSON envelope's note field). `Search` (reports which corpus it scanned and found nothing
in).

- test — `world ls --covers` with no hits names the phrase path
- test — `search --kind world` with no hits names the covers path
- test — a non-empty result carries no such line
- test — the JSON envelope carries the note as a field, not inside the hits
- test — neither line changes the exit code
