# Canonical render orders edges by module walk — renaming a file reorders the archive

**Kind:** bug (canonical form) · found while verifying the carrier-inference fix
**Status:** resolved 2026-07-08 — batch in surface order, migration v0005 (`renders-are-layout-blind`)

Deterministic lowering emits edges "in authoring order", and authoring order across files means
the sorted-module walk: the canonical render's edge block is a function of module *names*.
Renaming `src/operator.arch` to `src/agent.arch` — an identical model, bit-for-bit identical
semantics — moves every `Agent.*` edge from after the `Compiler.*`/`Engine.*` edges to before
them, the render hash changes, and `archi version current` reports `dirty: unsaved model changes
since v0004`.

## Observed

While closing plan `fix-carrier-inference`'s scenarios: carrier inference now survives the rename
(`archi check` compiles clean either way — `uses-see-every-def` holds), but the render diff is
±18 lines of pure edge reordering. Versioning's core claims break at the edges (literally):
"canonical bytes differ **iff** the model differs" and "a line diff between two renders is a
semantic diff" — here the model is unchanged and the diff is noise. A rename would mint a version
whose patch records nothing.

## Impact

File organization leaks into version identity: renames and file splits mint spurious versions (or
scare users with `dirty`), their patches bury real changes in reorder noise, and scope hashes
built over canonical subtrees may move without any semantic motion. The renaming-invariance
regression (`compilation_is_invariant_under_module_renaming`) had to stay single-edge to pass —
with two edge-carrying modules renamed across a sort boundary it fails today.

## Fix shape

Sort edge statements canonically in lowering (or at render): a total order over their canonical
surface text (type name, then endpoints, then carriers/views) — path/name-sorted like every other
statement class, so authoring order stops being load-bearing anywhere. Note this changes the
canonical form: every live model re-renders differently at the next save (one reorder-only
version per project), and the render contract has no version stamp to make that explicit — the
same ruler-vs-thing problem `hash-contract-is-versioned` solved for code links. Strengthen the
renaming-invariance test to multiple edge-carrying modules once fixed; then
`src/operator.arch` can finally become `agent.arch` (comment there points here).

## Resolution

Sorted in lowering, as the loop: `archi/stress/identity-pressure/` pressed v0004 — the phantom
replay (breaking) plus three survivors fencing the fix (`semantic-diffs-stay-semantic`: sorting
is a permutation, never a projection, and a total order makes one-edge diffs *smaller*;
`renders-still-compile`: every semantic precedence survives — classifiers within rank,
applications by delegation chain; `old-versions-stay-reconstructable`: reconstruction is byte
replay, seals never recompute against the new contract). Derived `renders-are-layout-blind`;
implemented via plan `sort-the-batch` @ v0004: lower.rs stages 5–7 emit rel edges by
(topological rank, surface), conn edges by surface, applications Kahn-ordered over the
attaches-outer-port relation with surface among the ready — the batch, render, scope slices and
`--emit-batch` all inherit it. Bonus the sort surfaced: delegation chains authored
inner-module-first used to fail compile (`NoOuterPort`) under adverse module names — chain order
fixed a latent correctness bug, pinned by
`delegation_chains_lower_outward_in_whatever_the_module_names`.

The migration went exactly as costed while the archive was five versions small: v0005 minted as
the reorder-only change (`version diff v0004 v0005` verified a pure permutation — 69 lines
moved, none added or removed), and `src/operator.arch` became `agent.arch` the same round with
`version current` unmoved at v0005. The renaming-invariance regression now spans two
edge-carrying modules and asserts byte-identical renders. Specs updated:
`source-format.md#lowering-and-determinism` (surface order, chain order, "identical *models*"),
`versioning.md` (the layout-blind claim and what changing the canonical form costs).
