---
links: [a-real-feature-file-meets-the-subset, the-first-fact-costs-an-hour, the-grammar-is-a-named-subset, Gherkin]
prefer: [simplicity, cost]
over: [operability]
---

# The grammar stays small

A level-three heading and four step keywords. Every other line under a heading raises a
located error naming what it found, and `Feature:` and `Scenario:` earn their own,
because the fact's title is the feature and the heading is the scenario.

Round one met a feature file that blocked the check and answered by reading the whole
language, so that a file brought from elsewhere would parse unedited. That was one
property bought with the largest single piece of implementation the world carries — a
parser with tables, outlines and their substitution rules — and six more rounds of
accumulation made the price the wrong one. The first author of a fact already faces more
work than anything else in this repository asks of them, and the parser sits on the critical
path to any of it working at all.

We pay in convenience. A feature file from another project is trimmed by the person
bringing it, and a team whose style leans on outlines writes the cases out. The error names
what it refused, so the trimming is mechanical.

A later round narrowed the shape again and this record follows it: the wrapping went too.
`Feature:` and `Scenario:` said twice what the fact's own title and the scenario's heading
already said, and the crate that read them left the manifest with them. What remains is a
heading and its steps, which is the smallest thing that can still be called Gherkin — and
still a strictly smaller grammar than the language, so nothing written here has to change
to reach it.
