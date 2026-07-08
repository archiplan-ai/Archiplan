---
affects: [Renderer, Archive]
outcome: breaking
---

# Rename mints a phantom version

A repository reorganizes its model sources — a module renamed to match its node
(`src/operator.arch` → `src/agent.arch`), a large module split in two. The model is bit-for-bit
identical; only file names moved.

## Attractor

Replayed from `issues/canonical-render-edge-order-depends-on-module-names.md`, hit while
verifying the carrier-inference fix: lowering emits edges and applications in authoring order,
and authoring order across files is the sorted-module walk — so the canonical render's edge
block is a function of module *names*. The rename moves every `Agent.*` edge across the sort
boundary, ±18 lines of pure reorder, the render hash changes, and `version current` reports
dirty on an unchanged model. A save would mint a version whose patch records nothing.
Versioning's core claims break at the edges, literally: "canonical bytes differ **iff** the
model differs" and "a line diff between two renders is a semantic diff" are both false the
moment file layout leaks into the bytes. The rename stayed blocked for three rounds because of
this — file organization held hostage by version identity.

## Resolution

Broke, as filed. Answered this round by finishing what the lowering spec already promises ("one
statement batch, in an order independent of authoring order"): rel edges sort by canonical
surface within their type's topological rank, conn edges by canonical surface, applications in
delegation-chain order (an application lowers after the application that attaches its outer
port) with canonical surface among the ready — so the batch, the render and the scope slices
are functions of the model alone. `src/operator.arch` became `agent.arch` the same round, and
`version current` did not move. Derived: renders-are-layout-blind.
