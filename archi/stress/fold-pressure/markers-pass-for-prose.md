---
affects: [StressDoc, DocsCompiler]
outcome: breaking
---

# Markers pass for prose

Two writers open a same-slug round on their own branches and merge under git's default
boundary. The add/add conflict wraps only the differing region — the two charters — in
markers; frontmatter and H1 merge clean.

## Attractor

The conflicted file is the detection surface, and nothing reads it: the chimera is green
before anyone resolves anything.

## Resolution

Broke ahead of where round nine placed the break. `two-rounds-one-record` recorded the
chimera appearing after the natural keep-a-charter resolution; the lab shows it needs no
resolution at all. `<<<<<<<`, `=======`, `>>>>>>>` land in the charter's prose region, the
markdown parser reads them as summary lines, and `check` exits 0 with the conflict still in
the working tree — the one moment the fused state is machine-obvious passes without a
diagnostic, while the same markers in `index.toml` get a recipe-naming `E_ARCHIVE`
(`remint-rejoins-the-lineage`). When the pins differ the markers land in frontmatter instead
and the diagnostic is a bare `E_DOC: frontmatter lines are key: value fields` — loud, but
naming a syntax accident rather than the merge state and its verb. Detection exists for one
store and not the other; the stress record is the one store whose fusion stays silent.
