---
links: [a-real-feature-file-meets-the-subset, the-first-fact-costs-an-hour, the-grammar-is-a-named-subset, Gherkin]
prefer: [simplicity, cost]
over: [operability]
---

# The grammar stays small

Six keywords and a tag line. `Background`, `Rule`, `Scenario Outline`, `Examples`,
docstrings and data tables raise a located error naming the construct.

Round one met a feature file that blocked the check and answered by reading the whole
language, so that a file brought from elsewhere would parse unedited. That was one
property bought with the largest single piece of implementation the wing carries — a
parser with tables, outlines and their substitution rules — and six more rounds of
accumulation made the price the wrong one. The first author of a fact already faces more
work than anything else in this repository asks of them, and the parser sits on the critical
path to any of it working at all.

We pay in convenience. A feature file from another project is trimmed by the person
bringing it, and a team whose style leans on outlines writes the cases out. The error names
the construct it refused, so the trimming is mechanical. If the subset turns out to bind in
practice, the whole language is a strictly larger grammar and nothing written under the
subset has to change to reach it.
