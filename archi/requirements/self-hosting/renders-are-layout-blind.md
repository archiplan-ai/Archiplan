---
kind: functional
origin: stressor(rename-mints-a-phantom-version)
satisfied-by: [Compiler.Lower, Renderer]
deferred:
---

# Renders are layout blind

The statement batch — and with it the canonical render and every scope slice — is a function of
the model alone: module names, file splits and authoring order never move a byte. Edges sit in a
total order over their canonical surface text like every other statement class (rel edges within
their type's topological rank, so classifiers still precede the shapes that consult them), and
applications order by their delegation chains — an application lowers after the application that
attaches its outer port — with surface order among the ready. Renaming or splitting `.arch`
files leaves `version current` untouched; only semantic change moves the hash.

## System Context

Versioning's core claims — "canonical bytes differ iff the model differs", "a line diff between
two renders is a semantic diff" — are only as true as the render is layout-blind. Replayed from
`issues/canonical-render-edge-order-depends-on-module-names.md`, which held the
`operator.arch` → `agent.arch` rename hostage for three rounds. Changing the canonical form
re-renders every live model once; the migration save records it as one reorder-only patch.

## Satisfy

`Compiler.Lower` (fixes the batch order: surface-sorted within every semantic precedence the
lowering encodes) and `Renderer` (emits the batch's creation order verbatim, so the bytes
inherit the invariance).

- test — source_e2e: two edge-carrying modules renamed across the sort boundary lower to a bit-identical batch and byte-identical render
- test — source_e2e: a delegation chain authored inner-module-first compiles — chain order, not authoring order, sequences applications
- test — existing semantic-change coverage: an added edge still moves the hash; the render round-trip (compile, re-render, byte-equal) stays green
- test — on this repository: the migration save's `version diff v0004 v0005` is a pure reorder, and the `agent.arch` rename leaves `version current` at v0005
