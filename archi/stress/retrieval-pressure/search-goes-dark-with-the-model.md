---
affects: [Search, Cli]
outcome: breaking
---

# Search goes dark with the model

Mid-refactor, the model doesn't compile — a half-renamed node, an unbound edge. The
agent, trying to orient itself in exactly that wreckage, runs `archi search` for the
requirement that names the invariant it is about to violate. If search compiles the
project the way `archi query` does and dies on the first diagnostic, retrieval is
unavailable precisely when the tree is broken — the moment it is worth the most.

## Attractor

Every read verb in the tool starts from a compiled model, so the reflex is to gate
search the same way. But search's corpus is mostly prose that never needed the
compiler, and the failure couples two independent corpora: one bad `.arch` line takes
the requirements, the stressors and the sessions dark with it. The agent that most
needs to find `renders-are-layout-blind` is the one who just broke the render.

## Resolution

Broke the all-or-nothing gate: each corpus degrades alone. A model that won't compile
darkens only the element cards — doc cards still scan, still rank, still answer — and
the response names the dark corpus with the first diagnostic instead of exiting.
A doc that won't parse degrades further: its raw text still matches, as a card with
no schema fields. Answered by `a-dark-corpus-stays-partial`.
