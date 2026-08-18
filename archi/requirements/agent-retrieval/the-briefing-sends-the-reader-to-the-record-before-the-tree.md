---
kind: functional
origin: intent
satisfied-by: [AgentBrief, Scaffold]
deferred:
---

# The briefing sends the reader to the record before the tree

The plan skill names `archi link ls --spec <node>` as the read that seeds a task's
`## Outputs`: the files already recorded against that node, instead of a guess from the
file names. The implement skill puts the same read in the sub-agent's prompt, for every
ref in the task's brief, so the writer starts from what is recorded and greps only for
what nothing names. And no skill sends a reader to review or confirm captured candidates,
because nothing captures them.

## System Context

`an-assistant-guesses-which-files-answer-a-written-obligation` says the assistant reads
the tree and guesses, and its first scenario says what should happen instead: it reads the
recorded files. The reverse view answers that question in one command and has for several
rounds. Every skill was written before it and none of them ask.

This is the third time this shape has cost a round: the tool grew an answer, the text that
would send somebody to it did not move, and the gap stayed invisible because a skill that
says nothing about a thing reads exactly like a skill that has nothing to say. The
`archi.md` failure-mode line still telling a blocked reader to "confirm or retire the
candidates" is the same defect, one release older, and the guard missed it because that
sentence names no command.

## Satisfy

`AgentBrief` (the plan skill's read before `## Outputs`, the implement skill's sub-agent
contract, and the failure mode that no longer exists). `Scaffold` (installs them
byte-equal to the embedded copies).

- test — the embedded plan skill names `archi link ls --spec` where it authors `## Outputs`
- test — the embedded implement skill's sub-agent contract requires the recorded files for
  the task's refs in the prompt
- test — no embedded skill tells a reader to confirm, review or retire a candidate
- test — the embedded workflow skill names `req ls --satisfies` in the passage that
  derives requirements
