---
kind: functional
origin: stressor(the-plan-never-meets-the-world)
satisfied-by: [Planner, WorldDoc]
deferred:
---

# A task carries the facts that cover its node

A task over a node names the world facts whose `covers` reaches that node, the way it
names the requirements it owns. `plan task show` lists them beside the requirements,
and `plan verify` flags a task whose named facts no longer resolve at the working
version. The facts are carried, not curated away: a task cannot drop one, because a
condition on the node's behavior is not the task's to waive.

## System Context

The world's scenarios are the only artifact in the spec that a run can decide, and
until now nothing brought them to the place where runs happen. The plan is that place:
it pins a version, it holds the tasks, and its waves are where an implementer reads
what the node has to do. A task that lists its covering facts puts the condition and
the Gherkin in front of whoever writes the code, at the moment they write it — which
is also the moment a link from a scenario to a step definition becomes something
somebody would author. Requirements are curated per task because ownership is a
choice; a covering fact is not owned, it applies.

## Satisfy

`Planner` (resolves the covering facts per task node at author time, carries them on
the task record, shows them beside the requirements and re-validates them on
`plan verify` and `plan repin`). `WorldDoc` (`covers` is the resolution key).

- test — a task over a covered node lists that fact on `plan task show`
- test — a task over an uncovered node lists none and passes
- test — a fact covering two of a plan's nodes appears on both tasks
- test — `plan verify` flags a task whose named fact was retired since the pin
- test — `plan repin` re-resolves the covering facts against the new version
