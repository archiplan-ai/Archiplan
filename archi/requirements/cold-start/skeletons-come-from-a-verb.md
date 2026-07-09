---
kind: functional
origin: intent
satisfied-by: []
deferred: init stands the project up; the per-document emitter waits until the cold start proves the emission discipline — one create-only verb first, then `archi new <kind> <name>` reusing it
---

# Skeletons come from a verb

`archi new intent|requirement|stressor|session <name>` emits a schema-perfect
skeleton — the slug-derived filename, every frontmatter field present and empty,
reserved sections in order, a placeholder summary line — so authoring a doc starts
at fill-in instead of write → check → fix.

## System Context

The doc schema is strict on purpose: every field present (empty is a state, absence
is not), a YAML subset with inline lists, fixed section order, an H1 that slugifies
to the filename. Each rule is individually good; together they are a gauntlet every
new file re-runs by hand, and `check`'s errors — however good — are still a loop
where a generator would be a step. The pain is recorded
(`issues/no-init-or-doc-scaffolding.md`); its doc-side half is not this round's
verb.

## Satisfy
