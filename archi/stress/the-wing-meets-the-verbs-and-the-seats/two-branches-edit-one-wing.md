---
affects: [Sessions, WorldDoc, DocsCompiler]
outcome: surviving
---

# Two branches edit one wing

Two seats work in parallel and both touch the wing: one edits a fact's scenarios, the
other adds `uses` pointing at it, a third file is retired on one side alone. Merge for
real. `archi-merge` triages with `check`, resolves version-archive collisions with
`remint` and folds concurrent stress rounds with `session fold`. The world wing has no
ceremony of its own: its files merge as text, and the `uses` graph they form is
reconstructed only when `check` next runs.

## Attractor

A merge that looks clean leaves a graph that is not. The retired fact is gone on one side
and named in `uses` on the other, so the error surfaces after the merge is committed, at
the desk of whoever merged, about a fact they never wrote. The repair is to guess which
writer was right about the world — the one judgement a merge seat is least able to
make.

## Resolution

Held, and for a structural reason rather than a lucky one. `session fold` exists because
a stress round keeps one charter file that two branches both write, so git fuses them
without a conflict. The wing has no shared file: one fact is one file, so a same-slug
collision is an add/add conflict git raises, and every other case merges as independent
files. What a resolution can break is the `uses` graph, and that is not silent either —
an unresolved `uses` is the blocking error `the-header-points-three-ways` already
defines, located at the line that names the missing slug. The merge seat meets one
conflict and one located list, never a quiet fusion.
