---
kind: functional
origin: intent
satisfied-by: [AgentBrief, Scaffold]
deferred:
---

# The sub-agent posts its declarations before it returns

The implement skill tells the task sub-agent to declare its own work as the last act of
the task: one `archi plan task <id> link add --symbol --answers --proved-by` for each
symbol it will defend, several of them through `archi batch -`. The rule that keeps every
`plan` and `link` command with the orchestrator holds for all other verbs and carves this
one out. The skill also stops sending the orchestrator to confirm captured candidates,
because capture proposes none. The orchestrator reviews the file, not the prose report.

## System Context

`the-writer-declares-what-the-code-answers` already says the sub-agent writes the
declaration, and it names the obstacle in its own text: the contract forbids the sub-agent
every `link` command. The verb landed. The contract did not move, so the orchestrator
files the declarations by hand and the refusal reaches the writer hours after the mistake,
which is the cost the verb was built to remove.

The skill is text beside the code, and nothing holds the two together. Three units changed
this loop and no skill file changed with them, so the shipped skill still sends its reader
to a list of candidates that capture no longer makes.

## Satisfy

`AgentBrief` (the implement skill carries the step, the batch form and the carve-out).
`Scaffold` (installs the skill byte-equal to the embedded copy).

- test — the embedded implement skill names the verb with all three flags
- test — the embedded implement skill names `archi batch -` in the sub-agent contract
- test — the embedded implement skill carves the verb out of the orchestrator-only rule
- test — no embedded skill names `link ls --evidence` or `link confirm`
