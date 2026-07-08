---
affects: [Compiler.Definitions]
outcome: breaking
---

# A bare modal needs no splice

The whole definition is one obligation clause — `port tokenize // must reject tabs in
indentation`. One sentence, no comma, well under 240 characters: the literal rule passes it
while it violates the design intent on its face.

## Attractor

Identity prose and obligation prose share a grammar; only the vocabulary differs. With no
second clause there is nothing to splice, and the single-sentence rule was written against
sprawl, not against content — the shortest possible definition is also the purest possible
obligation.

## Resolution

Broke: the pasted example passes every stated check and states no identity at all. Same answer
as the splice stressor — the vocabulary is the signal: must, should, shall, ensures and
handles reject wherever they stand in the prose. Answered by `obligations-never-define`.
