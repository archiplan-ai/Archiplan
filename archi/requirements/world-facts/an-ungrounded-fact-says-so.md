---
kind: functional
origin: stressor(the-agent-invents-the-fact)
satisfied-by: [WorldDoc, DocsCompiler]
deferred:
---

# An ungrounded fact says so

An empty `sources` reports `world_ungrounded`, whatever else the fact carries. The
earlier scoping fired only when `uses` was set as well, which missed the case that
matters most: a fact written whole, with no parent and no evidence, by an author who
observed nothing. The finding names the fact and never blocks.

## System Context

The world was designed on the assumption that a person saw something and wrote it down,
and the usual author is an agent. An agent asked for a condition about the world returns
a fluent one on demand: the shape is right, `covers` resolves, the scenario parses, and
nothing distinguishes it from an observation. `the-world-checks-form-and-never-truth`
accepted that no command decides truth, and that acceptance only holds while the tree can
at least say which facts nobody grounded. The count from `the-check-counts-the-world`
reports the aggregate; this finding names them one by one.

## Satisfy

`WorldDoc` (`sources` is the grounding, and its emptiness is a state the file wears).
`DocsCompiler` (emits `world_ungrounded` per fact with an empty `sources`, independent of
`uses`, `covers` or scenario count).

- test — a fact with empty `sources` and no `uses` reports `world_ungrounded`
- test — a fact with empty `sources` and `uses` set reports it once, not twice
- test — a fact with any source entry reports nothing
- test — the finding never changes the exit code
- test — the count line and the per-fact findings agree on the same set
