---
kind: functional
origin: intent
satisfied-by: [Gherkin, DocsCompiler]
deferred:
---

# Scenarios parse or the check fails

The `## Scenarios` block answers to one grammar. A `Feature:` line opens it. One or
more `Scenario:` blocks follow, and each block holds steps that start with `Given`,
`When`, `Then` or `And`, in that order of first appearance. A block that does not
parse raises a located `E_DOC` error with the line of the offending step. Free prose
inside the block is an error, because a scenario that a machine cannot read is a
paragraph wearing a costume.

## System Context

The scenario is the half of the unit a machine can hold. Its grain is free — a unit
check, an end-to-end run, anything that decides the behavior — but its shape is not,
because the shape is what a later `archi link` anchors to a step definition. That
link is the one edge from spec to code that a run decides instead of a person, and
it exists only while the steps are addressable. The grammar is deliberately the small
subset: `Feature`, `Scenario`, and the four step keywords. Tables, backgrounds,
outlines and tags stay out until a fact needs them.

## Satisfy

`Gherkin` (the grammar: a scenario block in, its steps or one located error out; the
subset is `Feature`, `Scenario`, `Given`, `When`, `Then`, `And`). `DocsCompiler`
(hands the `Scenarios` block of every world fact to the grammar during
`compile_docs`, and reports the error at the line inside the source file).

- test — a well-formed block of one Feature and two Scenarios parses to its steps
- test — a step that starts with none of the four keywords raises `E_DOC` at its line
- test — a `Scenario:` with no steps raises `E_DOC`
- test — a block with no `Feature:` line raises `E_DOC`
- test — the reported line is the line in the world file, not the line in the block
