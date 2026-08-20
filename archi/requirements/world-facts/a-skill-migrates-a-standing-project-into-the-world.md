---
kind: functional
origin: intent
satisfied-by: [Scaffold, AgentBrief]
deferred:
---

# A skill migrates a standing project into the world

The world pass of `archi-migrate` is what gives a standing project its world facts
(`one-door-migrates-the-standing-project` holds the door; this claim holds the pass). It reads
what the project already claims — the world prose inside its intents, and the free-text
story blocks its plans authored, and the tests its suites already name — and turns
candidates into facts one at a time, by interview. Its gate is the workaround: an operator
who cannot say what people do today instead of this has no condition, only a wish, and the
skill writes nothing. It asks by offering concrete options wherever answers have shapes,
because an open question phrased in the abstract stalls where the same question with two or
three candidate answers is answered at once. And a first "nothing would make this false" is
not a verdict: the skill asks again with concrete shapes, because a claim that looks like an
axiom is far more often a claim stated badly. Its output is
facts under `archi/world/` plus a brief naming what did not map and why. It deletes
nothing.

## System Context

Two earlier decisions leave a hole this skill fills and nothing else can. `the-intent-keeps-its-paragraph`
leaves world claims sitting in seventeen intents, reachable by no verb.
`the-old-plans-are-left-alone` leaves twenty-six story blocks unconverted, on the ground
that naming the condition under a story is judgement and a converter would have faked it.
Judgement is what a skill is: an operator runs it when they choose to, it asks questions, and
a person answers. A finding would have queued the same work as chores and been muted; a
converter would have produced fluent facts nobody observed — the failure
`the-agent-invents-the-fact` already named.

A migrated fact names the file it came from in `sources` — the intent, or the plan whose
story it carried. That entry resolves, so a migrated project reports nothing: swapping one
finding for another would defeat the purpose of running the skill at all. It is also true
in the plain sense of the field, which asks what the fact rests on: this one rests on a
document in the tree, and a reader sees at a glance that the ground is prose rather than an
observation.

## Satisfy

`Scaffold` (installs `.claude/skills/archi-migrate/SKILL.md` byte-equal to the
binary's embedded copy, as it installs the other skills). `AgentBrief` (the durable carrier
of the procedure: the interview, the workaround gate, the options it must offer instead of
open questions, the second ask on an apparent axiom, the test suites it reads for candidates,
and the brief it must return).

- test — `init` and `sync-skills` install the skill file byte-equal to the embedded copy
- test — the skill text names the workaround as the gate that stops a fact being written
- test — the skill text tells the reader to offer options rather than ask open questions
- test — the skill text tells the reader to ask again after a first "nothing would falsify it"
- test — the skill text names the test suites as a third place candidates come from
- test — the skill text requires a brief of what did not map
- test — a fact the skill mints names its origin file in `sources` and reports nothing
- test — a migrated project passes `check` with no world finding
- test — the skill deletes no intent prose and no plan story block
