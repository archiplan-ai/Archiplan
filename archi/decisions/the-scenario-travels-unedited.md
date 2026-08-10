---
links: [a-real-feature-file-meets-the-subset, the-grammar-takes-the-whole-language, Gherkin]
prefer: [operability]
over: [simplicity, cost]
---

# The scenario travels unedited

The grammar reads the whole of Gherkin — outlines, examples, backgrounds, rules,
tags, docstrings, data tables — and not the small subset the wing opened with.

The subset was the cheaper design and it was wrong for one reason. The wing joins a
fact to its scenario in one object so that the executable half travels with the reason
for it. A scenario that has to be trimmed on the way in, and trimmed again on the way
out, does not travel: it stays where the runner already reads it, the fact keeps a
prose sentence in its place, and the merged unit comes apart into the two documents it
was built to replace.

We pay for that in a real parser. A grammar with tables and outlines has cases, and
cases need tests and stay wrong in the meantime. This is the largest single piece of
work the wing carries, and it buys exactly one property: a feature file that runs
today parses here with no edit at all.
