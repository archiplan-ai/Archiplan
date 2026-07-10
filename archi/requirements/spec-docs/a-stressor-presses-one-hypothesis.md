---
kind: functional
origin: intent
satisfied-by: [DocsCompiler]
deferred:
---

# A stressor presses one hypothesis

One stressor file carries one pressure: a summary-first description of what presses; a
mandatory, non-empty `affects` list — the epistatic pressure surface, absolute paths
naming terms or types of the session's pinned version, types expanding to the terms they
classify when analyses run; an `Attractor` section for the state the stressor pulls the
system toward — what broken would look like; an `outcome` of `pending`, `surviving` or
`breaking`; and a `Resolution` that is non-empty exactly when the outcome is decided — a
verdict without its argument, or an argument without a verdict, is `E_DOC`. An affects
list can never be emptied (`E_AFFECTS_EMPTY`): a stressor that affects nothing is not a
stressor — delete the file instead. Affects stand whatever the outcome: they record where
pressure was applied, not how it went.

## System Context

The affects list is the join key of the incidence matrix
(`the-matrix-joins-stress-to-structure`); without it the cross-layer analyses would have
nothing to pivot on. Affects resolve against the pinned version, never the live tree
(`stress-pins-versions`), so widening a list mid-round is a text edit that stays
checkable forever.

## Satisfy

`DocsCompiler` (the stressor schema, the outcome/resolution coupling, non-empty affects,
and per-pin resolution of every path).

- test — docs::schema_violations_are_e_doc
- test — docs::affects_pin_to_the_sessions_version_while_satisfied_by_tracks_the_live_model
- test — incidence::a_type_expands_to_the_terms_it_transitively_classifies
