---
kind: functional
origin: stressor(the-hit-is-a-dead-end)
satisfied-by: [Search]
deferred:
---

# Cards carry the next hop

Every hit is a card, and every card carries the relations its kind already holds, as
slugs and paths ready for the next verb. An element card: its identity prose, the
requirements whose `satisfied-by` names it, the stressors whose `affects` names it,
and its edge neighbors. A requirement card: origin, satisfied-by and its
open/satisfied/deferred state. A stressor card: session, affects and outcome. A
session card: its pinned version and closing seal. An intent card is its address —
the folder under it is the hop.

## System Context

The scan already parses every machine field the schema defines; this requirement
forbids dropping them at the output boundary. The relational fields invert in the
same pass that reads them — requirement stamps and stressor affects index the element
cards they name — so the next hop costs nothing extra and the agent's follow-up
question is answered before it is asked.

## Satisfy

`Search` (cards typed per kind; the corpus pass inverts `satisfied-by` and `affects`
onto element cards; doc cards carry their parsed frontmatter relations and file:line
addresses; element cards carry model paths that pipe into `archi query --scope`).

- test — an element card carries its definition, the requirements stamped on it, the stressors pressing it and its neighbors
- test — a requirement card carries origin, satisfied-by and state; a stressor card carries session, affects and outcome
- test — doc cards carry file and line; element cards carry their model path
