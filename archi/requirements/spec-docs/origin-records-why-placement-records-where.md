---
kind: functional
origin: intent
satisfied-by: [DocsCompiler]
deferred:
---

# Origin records why, placement records where

Every requirement names where it came from: `intent` — derived from the enclosing intent,
legal only at the intent folder's root; `parent` — a pure refinement, legal only where a
parent requirement exists; `stressor(slug, …)` — the answer to one or more breaking
stressors; `fusion(slug, …)` — emerged at the junction of the named requirements. The
positional kinds are checked against placement (`E_PLACEMENT`), the slug kinds against
existence (`E_DOC_REF`), and the two axes stay orthogonal: a stressor-derived requirement
still lives somewhere in an intent's tree — its origin records why it exists, its path
records what it refines. Intent-origin requirements are never added mid-session:
mid-session requirements answer pressure, not new problem statements.

## System Context

Provenance is how a reader reconstructs the argument years later — which pressure bent
the design, which claims fused — and it stays trustworthy only if the build refuses
records that contradict their position (`breaking-derives-requirements` reads the
stressor side of the same join).

## Satisfy

`DocsCompiler` (parses the origin grammar, resolves every named slug, and cross-checks
the positional kinds against the tree).

- test — docs::placement_is_meaning
- test — docs::slugs_and_references_hold_project_wide
