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

A fact names no person either, and quotes nobody. It states how the world is, in ordinary
words, as a reader who was not in the room would state it — not that somebody disliked a
thing, not what somebody said, and never in their words. Who observed it is `sources`, and
their words are a file under `archi/world/resources/`. This half is not checked: no
mechanism separates a claim about the world from a claim about a person who spoke, and one
built out of phrase lists would fire on honest prose and miss the rest.

## System Context

This is the one content rule the world has, and it decides which world a claim belongs to.
A statement that cannot be made without the nouns of the model is a property of the
system, which is a requirement, killed by a test. A statement about people and their days
is a condition, killed by observation.

The second half falls out of the first. Attribution in the prose makes the fact about a
person's report rather than about the world: the reporter leaves and the sentence reads as
though the condition left with them, when the condition never depended on who noticed it.
It is also a third copy of what the record already carries — `sources` says who, and a
resource holds their words — and a field for information the record holds elsewhere is the
mistake this world has made more than once. Left in prose the rule erodes one sentence at a
time until the same obligation stands on both sides under two different killers. A finding
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
- test — a paragraph naming a person or quoting one is not reported: the rule is stated, not checked
