---
affects: [Agent, WorldDoc, DocsCompiler]
outcome: breaking
---

# The agent invents the fact

Ask an agent for a world fact. It produces a name, a conditioning paragraph, a killer
and a Gherkin block, all fluent, all plausible, and `sources` empty because it observed
nothing. `check` passes: the shape is right, `covers` resolves, the scenario parses. The
only signal is `world_inferred`, which fires on `uses` set and `sources` empty — and this
fact has no `uses` either, so nothing fires at all. A confabulated condition is
indistinguishable from an observed one by every mechanism the world has.

## Attractor

The world fills with facts that read like observations and were written by a language
model in one pass. They justify behavior, they carry scenarios, they enter plans. The
decision that check never verifies truth assumed a human observer behind each fact and
the tree does not know whether there was one. The world ends up the most authoritative
unverified surface in the repository, and its authority comes from the machinery around
it rather than from anything anybody saw.

## Resolution

The finding was scoped wrong and is rescoped: derived `an-ungrounded-fact-says-so`. An
empty `sources` reports `world_ungrounded` whatever else the fact carries, so the pure
invention — no parent, no evidence, written whole — is named where the old rule saw
nothing. This flags and does not prevent, which is the level
`the-world-checks-form-and-never-truth` already settled: no command decides truth, and the
acceptance holds only while the tree can say which facts nobody grounded.
