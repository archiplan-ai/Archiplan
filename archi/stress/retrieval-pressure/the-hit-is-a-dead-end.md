---
affects: [Search, SearchReport]
outcome: breaking
---

# The hit is a dead end

An agent searches "rate limiting", gets back `element Links.Capture 0.71` and a
snippet. Now what? To learn which requirements ride that element it runs a second
search, greps `satisfied-by` by hand, or reads three files whole — the retrieval verb
found the address and withheld the neighborhood, and every follow-up burns a
round-trip the phrase already paid for.

## Attractor

A hit list optimized for ranking alone treats the card as a pointer, but the agent's
next question is almost always relational: what satisfies this, what presses on it,
where did it come from, is it open. The KB holds every one of those answers as
machine fields the scan already parsed — dropping them at the output boundary forces
the agent to re-derive, with grep, relations the schema keeps for free.

## Resolution

Broke the pointer-only card: every hit carries its kind's own next hop, drawn from
fields the scan held anyway. An element card names its definition, the requirements
satisfied by it, the stressors affecting it and its edge neighbors; a requirement
card its origin, satisfied-by and open/satisfied/deferred state; a stressor card its
session, affects and outcome; a session card its pin and closing seal. Slugs and
paths, ready for the next verb. Answered by `cards-carry-the-next-hop`.
