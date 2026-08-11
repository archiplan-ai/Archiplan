---
affects: [WorldDoc, SourceTree, DocsCompiler]
outcome: breaking
---

# The source lives outside the tree

Write the fact that started this design: an operator learns of a collision late. The
grounding is a conversation, a support thread, a session recording, a ticket in another
system. `the-header-points-three-ways` resolves `sources` against the tree and makes an
unresolved entry an error, so none of those can be named. The honest author leaves
`sources` empty and their observed fact reads exactly like a guess.

## Attractor

`sources` collapses to a field that either points at a note somebody typed into the repo
to satisfy the checker, or stays empty. The first is a transcription of memory dressed as
evidence; the second erases the distinction the field exists for. Either way the one
signal the wing has for grounding stops carrying information, and the count of
sourceless facts — the instrument that watches the wing's health — measures the checker
rather than the world.

## Resolution

First answered by letting `sources` hold an external locator — a URI, a ticket id — kept
verbatim and never resolved. That answer was wrong and is retired. An entry nobody here can
open is a claim about evidence rather than evidence, and the field became unfalsifiable in
exactly the way the wing exists to prevent.

Answered instead by `a-source-is-reachable-and-lives-in-the-world`: the material comes into
`archi/world/resources/` or the field stays empty. Transcribing an interview is work, and
that work is the price of the field meaning anything. An empty `sources` says the true
thing — nobody has grounded this yet — and `world_ungrounded` reports it.
