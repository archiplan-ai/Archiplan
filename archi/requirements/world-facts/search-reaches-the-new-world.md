---
kind: functional
origin: intent
satisfied-by: [Search, Cli]
deferred:
---

# Search reaches the new world

The corpus that `archi search` scans grows by the world facts. One card is one fact,
built from its name, its conditioning paragraph and its killer. `--kind world`
narrows to them. A hit carries its slug and its `file:line` like every other card, so
the next command starts there.

## System Context

`one-verb-searches-everything` promises a ranked answer across every object the
knowledge base holds, so a new doc kind that search cannot see breaks a standing
requirement. The reach is real but the yield is bounded, and by construction: a world
fact is written without the nouns of the model, so its vocabulary and the vocabulary
of a query about the architecture barely meet. Lexical scoring cannot cross that gap.
Search finds a fact by the words of the fact. To go from a node to what conditions it
is the `covers` traversal, not a phrase.

## Satisfy

`Search` (one card per world fact from the same scan that reads requirements and
stressors; the scenario blocks stay out of the card body, because the steps are
addressed by `covers` and not by phrase). `Cli` (`--kind world` in the existing
narrowing flag; the JSON envelope carries the new kind).

- test — a phrase from a fact's name returns that fact as a hit with its slug and path
- test — `--kind world` returns world facts only
- test — the JSON envelope carries `world` as a hit kind
- test — a phrase that appears only inside a scenario step returns no world hit
