---
kind: functional
origin: intent
satisfied-by: [Gherkin, DocsCompiler]
deferred:
---

# Scenarios parse or the check fails

The `## Scenarios` block holds Gherkin and nothing else. `compile_docs` hands the
block to the grammar, and a block that does not parse raises a located `E_DOC` at the
line of the offending step inside the world file. The error blocks, as every `E_DOC`
does. Free prose in place of a scenario is an error, because a scenario a machine
cannot read is a paragraph wearing a costume.

## System Context

The scenario is the half of the unit a machine can hold. Its grain is free — a unit
check, an end-to-end run, whatever decides the behavior — but its shape is not,
because the shape is what a later `archi link` anchors to running code. That link is the
one edge from spec to code a run decides instead of a person, and it exists only while a
scenario parses and carries a name — `the-scenario-is-the-address-not-the-step` holds the
addressing. What the grammar accepts is the whole language
and is owned by `the-grammar-takes-the-whole-language`; this claim owns the blocking
and the location of the failure.

## Satisfy

`DocsCompiler` (hands the `Scenarios` block of every world fact to the grammar during
`compile_docs`, and reports the failure at its line in the source file). `Gherkin`
(returns the parsed steps, or one located error).

- test — a well-formed block parses and the check passes
- test — a malformed step raises `E_DOC` and the check exits non-zero
- test — the reported line is the line in the world file, not the line in the block
- test — a `Scenarios` heading holding prose instead of Gherkin raises `E_DOC`
- test — an empty `Scenarios` block raises `E_DOC`
