---
kind: functional
origin: stressor(the-read-envelope-skips-the-wing)
satisfied-by: [Query, Cli]
deferred:
---

# The read envelope carries the conditions

The read envelope answers with the facts covering the elements a request names: slug,
the conditioning statement and the scenarios, beside the model slice. `archi read` and
`archi query` both carry them. An element no fact covers answers with the slice alone.

## System Context

The wing exists so that whoever writes the code knows why the behavior is what it is, and
the usual reader is an agent working from the read surface. An agent that asks about a
node and receives nodes and edges will write against the happy path exactly as it did
before the wing existed — the conditioning would be written by agents and read by nobody.
`world ls --covers` answers the same question for a person with a node in hand; the
envelope answers it for the reader who did not know to ask.

## Satisfy

`Query` (resolves the covering facts for the elements in a composed slice and carries
them in the answer). `Cli` (`read` and `query` render them in the human and JSON forms).

- test — a slice naming a covered element carries that fact's statement and scenarios
- test — a slice naming an uncovered element carries the slice alone
- test — the JSON envelope carries the facts under their own key
- test — a fact covering two elements of one slice appears once
- test — a tree with no wing answers exactly as it did before
