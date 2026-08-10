---
kind: functional
origin: stressor(a-real-feature-file-meets-the-subset, the-first-fact-costs-an-hour)
satisfied-by: [Gherkin]
deferred:
---

# The grammar is a named subset

The grammar reads six keywords and tags: `Feature`, `Scenario`, `Given`, `When`, `Then`,
`And`, plus a tag line above a feature or a scenario. Everything else in the Gherkin
language — backgrounds, rules, scenario outlines, examples tables, docstrings, data
tables — raises a located error naming the construct and the subset. A feature file
brought from elsewhere is trimmed to the subset by the person bringing it.

## System Context

Round one met this pressure and answered it by reading the whole language, which bought
one property — a foreign file parses unedited — for the price of a real parser with
tables, outlines and their substitution rules. Seven rounds of accumulation later that
price is the wrong one: the wing already asks more of its first author than anything else
in this repository, and the parser is the largest single piece of work in it. The subset
is small enough to hold in a head and to implement once. Tags stay, because
`a-scenario-names-where-it-runs` puts the runner on one, and a tag line costs nothing to
parse. The error names the construct it refused, so trimming a foreign file is mechanical
rather than a guessing game.

## Satisfy

`Gherkin` (the six keywords, the tag line, and a located error for every other construct,
naming what it found and what the subset holds).

- test — a block of `Feature`, two `Scenario`s and their `Given/When/Then/And` steps parses
- test — a tag above the feature and above a scenario parses and is kept
- test — `Background`, `Rule`, `Scenario Outline`, `Examples`, a docstring and a data table
  each raise a located error naming the construct
- test — the error names the six keywords the subset holds
- test — `But` and `*` raise the same error as any other unknown step keyword
