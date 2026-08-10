---
kind: functional
origin: stressor(a-real-feature-file-meets-the-subset)
satisfied-by: [Gherkin]
deferred:
---

# The grammar takes the whole language

The grammar reads Gherkin as a runner reads it, not a subset of it: `Feature`,
`Background`, `Rule`, `Scenario`, `Scenario Outline` with its `Examples` tables, the
step keywords `Given`, `When`, `Then`, `And` and `But`, tags above any of them,
docstrings and data tables inside a step, and comment lines. A feature file that a
runner executes today parses here unchanged.

## System Context

The wing merges the fact and its scenario into one object so that the executable half
travels with the reason for it. That only holds while a scenario can be moved in from
where it already runs, and out to where it will run, without an edit. A subset makes
the wing a place to copy out of: the scenarios drift to the code tree, the fact keeps
a sentence in their place, and the merged unit comes apart. Reading the whole language
costs a real parser to keep correct, and that price is recorded.

## Satisfy

`Gherkin` (the full grammar over one scenario block: keywords, tags, outlines and
their examples, docstrings, data tables and comments; an unparsable block yields one
located error).

- test — a feature file with `Background`, a `Scenario Outline` and its `Examples`
  table parses unchanged
- test — tags above the feature and above a scenario parse and are kept
- test — a docstring and a data table inside a step parse as that step's payload
- test — `But` and `*` are read as step keywords
- test — a scenario outline with an examples table of three rows yields three cases
- test — a genuinely malformed block still yields one located error, not a silent pass
