---
kind: functional
origin: intent
satisfied-by: [DocsCompiler]
deferred:
---

# An internal element says so

`archi/world/.worldignore` names the elements that no condition outside will ever reach.
One line is one element path and the reason it is internal, separated by ` — `. `check`
resolves every entry against the model: an entry that names nothing is a located error, and
an entry with no reason is a located error. A named element reports no `world_unreached`.
Data-classified elements are excluded by their type and never belong in the file.

## System Context

Coverage without a floor is a finding that never empties, and a finding that never empties
stops being read — `the-findings-crowd-the-worklist` already showed that shape. Requirements
have `deferred: <reason>` for exactly this: a claim that stands open, on the record, with the
reason attached. The wing had no equivalent, so an element that will never carry a condition
had no honest way to leave the list, and the only way to silence it was to invent a fact
about it.

The file resolves rather than matches by string, which is what separates it from a glob list
in the manifest: a renamed element breaks it loudly and the operator repairs it, the same
contract `covers` carries. And it is a claim, not a mute — "no behavior from outside reaches
this" is a statement somebody can be wrong about, so the reason is mandatory and a reader can
argue with it.

## Satisfy

`DocsCompiler` (reads the file beside the wing, resolves each entry against the compiled
model, errors on an unresolved entry and on a missing reason, and suppresses
`world_unreached` for the named elements).

- test — an entry naming a model element suppresses its `world_unreached`
- test — an entry naming nothing raises a located error
- test — an entry with no reason raises a located error
- test — a Data-classified entry raises a located error, because its type already excludes it
- test — a tree with no `.worldignore` behaves exactly as it does today
- test — every element either carries a condition, is Data, or is named here: the count reaches zero
