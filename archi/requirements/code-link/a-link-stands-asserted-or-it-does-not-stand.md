---
kind: functional
origin: intent
satisfied-by: [Links, Links.Grader, Links.Journal]
deferred:
---

# A link stands asserted or it does not stand

A link has one standing. `archi link confirm` retires with the concept, and so do the
`--evidence` filter, the confidence score, its erosion by repeated waves, the decay event
and the `decayed_evidence` finding. A journal that carries decay events from an older
binary still folds: the events are read and skipped, never an error. A row that was minted
as evidence loads as asserted and grades like every other row.

## System Context

The second standing existed for one producer: capture minting from a shared word. The
producer is gone, this project holds no evidence row, and the machinery around it grades
nothing, scores nothing and decays nothing. A verb with no live input is worse than absent,
because it appears in the help and answers the reader with silence.

Removing it and leaving the rows is the honest trade. What was a guess still reads as a
guess in `rule`, which is the record of how it was born, and nothing in the tree can promote
one — a claim worth keeping is written again with `link add`.

## Satisfy

`Links` (one standing, no confirm verb, no `--evidence` filter). `Links.Grader` (no
confidence, no decay, no `decayed_evidence` in the audit). `Links.Journal` (a decay event
from an older writer folds and is skipped, and an evidence row loads as asserted).

- test — `link confirm` and `link ls --evidence` are gone from the surface: both answer a usage error
- test — the audit reports dark code and dark spec, and never a decayed row
- test — a journal holding an evidence add and a decay event folds without error, and the row lists as asserted
- test — `link verify` grades that row exactly as an asserted one
