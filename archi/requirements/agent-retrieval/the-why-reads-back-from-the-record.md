---
kind: functional
origin: intent
satisfied-by: [AgentBrief, Scaffold]
deferred:
---

# The why reads back from the record

`archi-explain` is the page for "why is this the way it is". It is read-only: the answer
comes from the record, never from speculation. The chain runs world first — the condition
the behavior serves (`world ls --covers`), then the claims that must hold
(`req ls --satisfies`), then the recorded trade-offs (`decision ls --links`), then each
requirement's origin back to its stressor, then the timeline (`version list`,
`version diff` — the tree never moves), then who realizes it today (`link ls --spec`).
The answer leads; decisions are quoted verbatim with the alternatives that lost; every
citation carries its address. A question with no recorded trade-off is answered "the
record holds no rationale here" and an offer to record one — silence is a real answer,
and invented rationale is forbidden in as many words.

## System Context

The old client carried this discipline in `archi-explain` and the port died with it: the
new tool writes the why harder than the old one ever did — decisions price their trades,
stressors sign their breaks, saves carry their reasons, the world holds the conditions —
and no page taught reading it back. The write side without the read side is a diary
nobody opens.

The chain starts at the world because that layer did not exist when the old page was
written, and it answers the deepest form of the question: not "which trade shaped this"
but "which outside condition makes this behavior necessary at all".

## Satisfy

`AgentBrief` (the `archi-explain` page: the chain, the answer rules, the read-only
mandate, the `archi-search` pointer for resolving a phrase to an address). `Scaffold`
(embeds and installs the page byte-equal, like the rest).

- test — a fresh init installs `archi-explain` byte-equal to the embedded copy
- test — the embedded page orders the chain: world, then requirements, then decisions,
  then stressors, then versions, then links
- test — the page says silence is a real answer and forbids invented rationale
- test — the page is read-only in as many words and mutates nothing
