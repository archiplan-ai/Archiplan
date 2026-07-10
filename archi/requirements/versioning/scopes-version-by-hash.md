---
kind: functional
origin: intent
satisfied-by: [Archive, Renderer]
deferred:
---

# Scopes version by hash

Scope versions are derived from whole-model versions, never stored independently: every
node carries a Merkle-style hash over its canonical subtree, a scope's version identity
is that hash, and its history is the sequence of distinct hashes across saved versions —
vertical propagation holds by construction, a change anywhere under a node moving every
ancestor's hash and no sibling's. Two hashes per scope turn "does an internals change
bump the outer version?" into consumer policy rather than storage: the full hash covers
everything under the node; the interface hash covers the declared ports plus the boundary
edges — edges with exactly one end inside the subtree. An internals-only change moves the
full hash and leaves the interface hash. Root-scope hashes are recorded per version in
the manifest, so "which versions touched X" is a manifest scan; deeper scopes reconstruct
and hash on demand.

## System Context

One storage mechanism means no coherence problem between scope and system versioning, and
the full/interface split is the precedent the code-link hashes follow —
`drift-graded-per-kind` watches the same distinction in code.

## Satisfy

`Renderer` (slices the canonical render per root and hashes subtrees). `Archive` (records
root-scope hashes in each manifest entry).

- test — versions::scope_hashes_split_interface_from_internals
- test — canonical::scope_sources_slice_the_render_per_root
