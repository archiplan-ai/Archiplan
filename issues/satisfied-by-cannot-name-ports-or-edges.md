# `satisfied-by` can name only nodes and types — not ports or edges

**Kind:** limitation (requirements expressiveness) · found by the self-hosting bootstrap

A requirement's `satisfied-by` entries validate via `Model::has_node`
(`crates/modeling-lang/src/model.rs:415`), which descends node children only. Ports
(`Engine.answer`) and typed edges do not resolve, so a claim that honestly pins to an interface
point or a connection must be laundered through the owning node.

## Observed

`agents-read-lowered-statements` claims "the envelope is read-only; a write smuggled into it is
rejected" — the element that satisfies this is the `Engine.answer` port (or the `interrogate`
edge into it), but the requirement can only say `satisfied-by: [Engine, Query]`. Meanwhile the
*link* layer accepts both node paths and canonical edge surface text as `SpecRef`s
(`crates/archi/src/links/mod.rs:120`), so spec↔code traceability is finer-grained than
spec↔requirement satisfaction — an asymmetry with no apparent reason.

## Impact

Satisfaction claims over-approximate: a requirement about one port of a hub node reads as a claim
about the whole node, which inflates the invariant surface incidence uses and dulls the
reverse-lookup that seeds plan requirements.

## Fix shape

Let `satisfied-by` accept what links already accept: port paths and canonical edge text. The
expansion machinery exists — `term_surface` handles types, and plans already match edge refs
through their endpoints (`plans::matched_requirements`).
