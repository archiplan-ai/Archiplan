---
kind: functional
origin: stressor(the-rule-against-model-nouns-is-prose)
satisfied-by: [WorldDoc, DocsCompiler]
deferred:
---

# The fact speaks the world and check says when it does not

The fact statement and its conditioning paragraph name no element of the model. `check`
holds every element name of the compiled graph, so it reports `world_speaks_the_model`
naming the fact and the element it found. The scenarios are exempt: Gherkin describes the
system's surface and naming it there is the point.

## System Context

This is the one content rule the wing has, and it decides which wing a claim belongs to.
A statement that cannot be made without the nouns of the model is a property of the
system, which is a requirement, killed by a test. A statement about people and their days
is a condition, killed by observation. Left in prose the rule erodes one sentence at a
time until the same obligation stands in both wings under two different killers. A finding
rather than an error, because a fact may legitimately quote a name — and because the
judgement of whether it had to stays with the reader.

## Satisfy

`DocsCompiler` (matches the compiled element names against the fact's name and
conditioning paragraph, skipping the `Scenarios` block, and emits the finding with the
element it matched). `WorldDoc` (the exempt region is exactly the scenario blocks).

- test — a paragraph naming a model element reports `world_speaks_the_model`
- test — the same name inside a scenario step reports nothing
- test — the finding names the matched element
- test — a fact naming no element reports nothing
- test — the finding never changes the exit code
