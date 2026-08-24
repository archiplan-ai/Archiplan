---
kind: functional
origin: stressor(a-real-feature-file-meets-the-subset, the-first-fact-costs-an-hour)
satisfied-by: [Gherkin]
deferred:
---

# The grammar is a named subset

A scenario is a level-three markdown heading and the step lines under it. The heading is
the scenario's name, and its address. A step line opens with `Given`, `When`, `Then` or
`And` and holds its text; those four are the whole vocabulary. Any other line under a
heading is a located error naming what it found. `Feature:` and `Scenario:` are named
explicitly by that error, because they are the shape this replaced: the fact's own title
is the feature, and the heading is the scenario, so repeating either says the same thing
twice.

## System Context

Round one met this pressure and answered it by reading the whole language, which bought
one property — a foreign file parses unedited — for the price of a real parser with
tables, outlines and their substitution rules. Seven rounds of accumulation later that
price is the wrong one: the world already asks more of its first author than anything else
in this repository, and the parser is the largest single piece of work in it. The subset
is small enough to hold in a head and to implement once. Tags stay, because
`a-scenario-names-where-it-runs` puts the runner on one, and a tag line costs nothing to
parse. The error names the construct it refused, so trimming a foreign file is mechanical
rather than a guessing game.

## Satisfy

`Gherkin` (the heading-and-steps reader: a level-three heading opens a scenario, the four
step keywords open its lines, and every other line is a located error naming what it found
and where that thing belongs).

- test — two headings and their step lines parse into two named scenarios
- test — a `Feature:` line raises a located error naming the fact's title as its place
- test — a `Scenario:` line raises a located error naming the heading as its place
- test — prose under a heading raises a located error naming the four step keywords
- test — `But` and `*` raise the same error as any other unknown step keyword
- test — a heading with no step under it raises a located error
- test — a `## Scenarios` block with no heading raises a located error
- test — two headings sharing a name inside one fact raise a located error
