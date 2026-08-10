---
kind: functional
origin: stressor(the-wing-outgrows-the-reading)
satisfied-by: [DocsCompiler]
deferred:
---

# The check counts the wing

`check` closes with one line for the wing: how many world facts stand, and how many
of them carry an empty `sources`. The line prints on every run, with no threshold and
no verdict. It adds no field to the header.

## System Context

The wing catches a fact that stopped being true in one way only — a person reads it
again and says so. That works while the wing is small enough to read in one sitting,
which was the assumption the whole design rested on and which no file stated. A count
is the cheapest thing that makes the assumption visible: an operator who sees the
number climb knows the reading is no longer happening, without any machinery that
claims to know whether a fact is true. The ungrounded half of the count is the part
that matters most, because a fact with no sources was never anchored to anything.

## Satisfy

`DocsCompiler` (counts the loaded world facts and the subset with an empty `sources`,
and emits the line beside the scoring line at the close of a passing check).

- test — a tree with seven facts, two of them without sources, prints both numbers
- test — a tree with no world facts prints no line
- test — the line prints on a passing check and on one that reports findings
- test — the line never changes the exit code
