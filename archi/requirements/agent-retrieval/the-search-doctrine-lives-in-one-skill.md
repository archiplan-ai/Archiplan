---
kind: functional
origin: intent
satisfied-by: [AgentBrief, Scaffold]
deferred:
---

# The search doctrine lives in one skill

`archi-search` is the one page that says how to find things: the semantic menu first
(`query --top`), then the structural reads from an element (`req ls --satisfies`,
`world ls --covers`, `link ls --spec`), search last — when you hold a phrase and no
address — and grep never, with the reason. Every working skill carries one pointer line
naming `archi-search` and no retrieval vocabulary of its own; the workflow steps keep the
commands they use at the moment they use them. The doctrine's distinctive sentences exist
in exactly one file, so there is no second copy to drift.

## System Context

The retrieval order was written into one skill's ground rule, and six other skills that
search never saw it. The alternative — the same block repeated per skill — is the disease
this tree spent a week curing: copies drift silently, and a guard on byte-equality of
seven copies is machinery for a problem the single copy does not have.

The pointer carries no summary deliberately. A pointer with a digest is a copy again,
and the digest is what drifts. The skills already reference each other by bare name —
planning is `archi-plan`, closing is `archi-finish-worktree` — and install together, so
the referenced page is always beside the reader.

## Satisfy

`AgentBrief` (the `archi-search` page, the pointer line in every working skill).
`Scaffold` (embeds and installs the tenth skill byte-equal, like the other nine).

- test — a fresh init installs `archi-search` byte-equal to the embedded copy
- test — the embedded `archi-search` names the order: `query --top`, then `req ls
  --satisfies`, `world ls --covers`, `link ls --spec`, then search, and says grep misses
  the model
- test — every working skill names `archi-search`, and `ste-writing` is exempt
- test — the phrase `Search, do not grep` and the doctrine's order live only in
  `archi-search`
