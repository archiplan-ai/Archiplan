---
kind: functional
origin: intent
satisfied-by: [WorldDoc, DocsCompiler]
deferred:
---

# A world fact carries its scenarios

One file under `archi/world/facts/` holds one fact about the world together with every
scenario that fact dictates. The name is the fact in one line. The paragraph after it
states the outside condition and why the behavior follows from it. `## What people do
instead` names the workaround and what it costs. `## Scenarios` holds one or more
scenarios, each a level-three heading naming it and the step lines beneath. `## Open
questions` is optional and holds what is not known yet. The fact is the key of the file:
when it goes, the file goes, and its scenarios go with it.

## System Context

A requirement is a property of the system and a test decides it. A world fact is a
condition outside the system and only observation decides it. The two answer to
different killers, so they are separate objects, and the divider is origin and not
the grain of the test a scenario carries. Pairing the fact with its scenarios in
one file keeps the reading honest: a scenario without its condition freezes into
dogma, and a condition without its scenarios is a wiki page. One file, two objects
in the graph — the scenario carries the state of its run, the fact carries its
sources, and `check` never mixes them.

The workaround is a section of its own because it does two jobs no other part can. It
is the gate: a condition nobody can name a workaround for is a wish, and a wish belongs
in no file. Dissolved into the paragraph, a fact written without one looked exactly like
a fact written with one, so the gate rested on whoever was typing. It is also the
falsification test — watch whether people still do it, and the day they stop, the fact
is dead. That is an observable to check against, where a written prediction of the end
was a guess made at the worst moment for guessing.

## Satisfy

`WorldDoc` (the markdown object: the fact as the name, the conditioning paragraph, the
workaround, the scenarios and the optional open questions). `DocsCompiler` (holds the
shape: a missing name, a missing paragraph, a missing workaround or an empty scenario
list is a located error, and the open-questions heading stays optional).

- test — a file with name, paragraph, workaround and one scenario passes
- test — a missing conditioning paragraph, a missing `What people do instead` and an
  empty `Scenarios` each raise a located `E_DOC`
- test — a `What kills this` heading raises a located error naming what replaced it
- test — a file with no `Open questions` heading passes, and one with an empty
  `Open questions` heading passes
- test — any other heading in the file is kept and reported by no error
