---
kind: functional
origin: intent
satisfied-by: [Search]
deferred:
---

# A node with no prose speaks through its ports

A node that carries no definition of its own is indexed by its ports' definitions at summary
weight, in the slot its missing prose would have filled. A node that carries its own
definition is untouched: its ports' prose stays at body weight, where it is supporting
detail beside a statement that already exists.

## System Context

The search scores three fields — name, summary, body — at 3, 2 and 1. A node's own
definition is the summary; its ports' definitions are body. So a node with no definition
does not merely lack prose, it forfeits a whole weight class, and any node that happens to
carry a sentence outranks it on a word neither of them is really about.

That is not a rare corner. Of 54 top-level nodes here, every one of the 30 that carries a
definition is portless, and not one of the 24 that carry ports carries a definition. The
split follows the grammar: a portless node is one line and a comment lands at its end, while
a ported node opens a block with a colon and there is no end to land on, so the author
describes the ports instead. The nodes with ports are the ones that do something. Asking the
index who does a thing therefore answers with what is lying around.

Promoting the ports is the honest reading rather than a workaround: for a node whose only
prose is its ports', that prose *is* the statement of what it is. Promoting every node's
ports would be the workaround — a node with many ports would then float on volume alone.

## Satisfy

`Search` (indexes a definitionless node's port definitions as summary, and leaves a defined
node's ports as body).

- test — a node with ports and no definition ranks on its port prose at summary weight
- test — the same node loses to a name match, which still outweighs a summary
- test — a node that carries its own definition keeps its ports at body weight
- test — a node with no ports and no definition scores on its name alone, as it does today
- test — the card of a definitionless node shows its port prose, as it does today
