---
kind: functional
origin: intent
satisfied-by: [Links, Links.Journal]
deferred:
---

# A link is fact plus projection

A link carries two layers of opposite mutability. The birth record is an immutable
provenance fact — these spans, in this delta, were written to realize this spec element,
under this task — storing content, not references: file, span, span-content hash, the
symbol resolved at capture, the canonicalizer version; a commit sha is optional
provenance, never a dependency, for the same reasons versions refuse git as a store. The
projection is where that code lives now — the resolved symbol and its hashes — derived,
recomputed by verify, cached at most, never authored. The spec side is a SpecRef: a node
path or an edge's canonical surface text, in a version slot — a pinned `vNNNN` or
Working, the live tree.

## System Context

A pin asserts a present-tense correspondence and decays with every commit; a fact about a
delta never drifts. Splitting relocates the fragility into a computation —
`link-truth-is-append-only` holds the journal the facts land in, and
`hash-contract-is-versioned` keeps the stored hashes honest across canonicalizer
changes.

## Satisfy

`Links` (mints birth records at add and capture, recomputes projections at verify).
`Links.Journal` (the record's one store; projections never write back into it).

- test — links::add_verify_and_the_drift_grades
- test — links::spec_refs_resolve_nodes_edges_and_slots
- test — links::refs_parse_their_member_and_render_it_back
