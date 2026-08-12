---
kind: functional
origin: intent
satisfied-by: [Archive, DocsCompiler]
deferred:
---

# The save refuses an unconditioned element

`archi version save` refuses on a tree that holds at least one world fact and still carries
an element that no fact reaches: not named in a `covers`, not reachable from one along the
declared edges, not a child of a covered element, not classified as `Data`, and not declared
internal in `archi/world/.worldignore`. The refusal names every such element and the two
ways to clear it — cover it with a condition, or declare it internal with a reason. A tree
with no facts saves as it always did.

## System Context

`world_unreached` already computes this set and reports it as a finding. A finding is
advisory by design, so the gap it names travels: the version saves, the plan is authored
against it, the waves run, and the gap surfaces at the closing block — where the fix is a
world fact, which is spec work, which means walking back through the plan to the spec
stage. That walk back is the failure. It happened on this very unit.

The moment to decide whether an element is conditioned by the world or is internal
machinery is the moment the element was drawn, and the person who drew it holds the answer.
A gate at the save asks them then. A finding asks nobody, ever.

The declaration is what keeps the gate honest rather than merely loud: half of any model is
a lexer, a canonicalizer, a registry — machinery no condition outside will ever reach, and
`an-internal-element-says-so` is where that is said, once, with a reason a reader can
argue with. So the gate has two exits and neither is a lie: name the condition, or name the
element as internal.

It bites only where the world already stands. A project that has not written its first fact
saves exactly as before, which is the same threshold every other rule in the world uses.

## Satisfy

`DocsCompiler` (computes the unreached set it already computes, and hands it to the save as
a refusal rather than a finding). `Archive` (refuses the save, names every element and both
exits, and changes nothing on a tree with no facts).

- test — a tree with one fact and one unreached element refuses the save, naming the element
- test — the refusal names both exits: cover it, or declare it internal
- test — covering the element clears the refusal and the save proceeds
- test — declaring the element in `.worldignore` clears it as well
- test — a `Data`-classified element never blocks a save
- test — a tree with no world facts saves exactly as it did before
- test — `check` still reports the same set as a finding, so the two never disagree
