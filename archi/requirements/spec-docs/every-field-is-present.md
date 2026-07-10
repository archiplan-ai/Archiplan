---
kind: non-functional
origin: intent
satisfied-by: [DocsCompiler]
deferred:
---

# Every field is present

Empty is an explicit state; absence is ambiguity, and ambiguity does not compile. A
requirement file carries all four machine fields — `kind`, `origin`, `satisfied-by`,
`deferred` — possibly empty, plus the reserved sections in fixed order after the
summary-first body; a stressor carries `affects` and `outcome` with `Attractor` then
`Resolution`; a session carries `version` and `closed`. A missing or unknown field or
section, misordered sections, malformed frontmatter, or a half-present satisfaction
record — elements without prose, or prose without elements — is `E_DOC` at the offending
file and line. Machine fields stay plain YAML lists and scalars, so `check`, indexers and
`grep` read them without parsing prose, and prose never has to encode data.

## System Context

The documents are written mostly by agents, and a schema that tolerated absence would
turn every reader into a guesser. The strictness is a gauntlet by design; the scaffolding
that would ease authoring is its own deferred verb (`skeletons-come-from-a-verb`).

## Satisfy

`DocsCompiler` (parses every doc against its kind's schema and rejects deviations as
located `E_DOC`).

- test — docs::schema_violations_are_e_doc
- test — md::structural_errors
