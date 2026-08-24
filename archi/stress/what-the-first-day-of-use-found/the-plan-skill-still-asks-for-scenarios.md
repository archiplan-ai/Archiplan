---
affects: [PlanFile, Planner, AgentBrief]
outcome: breaking
---

# The plan skill still asks for scenarios

Author a plan through `/archi-plan` after the world landed. The skill still says to walk
the architecture as a user and write one bullet per flow into `scenarios.md`. The author
does it — six sentences, carefully written. Then `plan scenarios list` answers with the
block collected from the world facts, and the hand-written file is read by nobody, ever.
This happened on the very first plan authored after the change.

## Attractor

Two artifacts named the same thing, one of them dead, and the dead one is the one the
procedure asks for. Every plan authored from here carries a file that looks like the
closing block and is not it, and a reader who opens the folder finds the wrong one first.
The rule that the plan authors no scenarios lives in a requirement nobody reads while
planning, and the procedure everybody follows says the opposite.

## Resolution

The procedure moves with the behaviour: derived
`the-plan-collects-its-scenarios-and-authors-none`. The planning skill names the world
facts as the source of the closing block, says what an empty block means, and gives no
instruction to author one. The requirement had already settled the tool's half; this
settles the person's.
