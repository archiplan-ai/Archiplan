# Canonical render orders edges by module walk — renaming a file reorders the archive

**Kind:** bug (canonical form) · found while verifying the carrier-inference fix

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
