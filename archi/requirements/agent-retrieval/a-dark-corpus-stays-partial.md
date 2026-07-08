---
kind: non-functional
origin: stressor(search-goes-dark-with-the-model)
satisfied-by: [Search, Cli]
deferred:
---

# A dark corpus stays partial

Search never dies whole. A model that fails to compile darkens only the element cards:
doc cards still scan, rank and answer, and the response carries a `dark` note naming
the model corpus with its first diagnostic. A doc file that fails the markdown parse
degrades to a raw-text card — its lines still match, its schema fields stay empty.
The verb exits zero with whatever it could search; diagnosing the breakage stays
`archi check`'s job.

## System Context

Search is the orientation verb for a broken tree — mid-refactor, mid-merge — so it
inverts the tool's usual gate: where `query` and `read` refuse on a compile error,
search treats the compiled model as one corpus among several and each corpus fails
alone. The precedent is `plan verify`, which reads a best-effort doc tree and reports
doc diagnostics as notes.

## Satisfy

`Search` (per-corpus degradation: the element corpus is absent when the compile fails,
each unparseable doc falls back to its raw lines). `Cli` (compiles the project without
treating diagnostics as fatal for this verb, threads the dark note into both output
forms, exits zero).

- test — with a non-compiling model, doc hits still return and `dark` names the model corpus with a diagnostic
- test — a doc file that fails the markdown parse is still found by a phrase from its raw text
- test — the broken-model search exits zero; `archi check` on the same tree still exits one
