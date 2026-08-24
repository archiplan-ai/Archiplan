---
kind: functional
origin: stressor(the-plan-skill-still-asks-for-scenarios)
satisfied-by: [AgentBrief, Scaffold]
deferred:
---

# The plan collects its scenarios and authors none

The planning procedure says the plan authors no scenarios. It names where the closing
block comes from — the world facts covering the nodes its tasks hold — and what an empty
block means: nobody has recorded a condition about those nodes, which is work for the spec
stage and not a blank to fill. It never asks the author to write `scenarios.md`.

## System Context

`a-plan-s-own-scenarios-block-retires` settled the behaviour and the tool obeys it. The
procedure everybody follows was not changed with it, so the first plan authored after the
landing carried six hand-written sentences that no verb reads. A rule that lives only in
a requirement is a rule the planner meets after they have already done the wrong thing.

Behaviour and procedure have to move together. The requirement says what the tool does;
the skill says what the person does; when they disagree the person wins, because they are
the one holding the keyboard.

## Satisfy

`AgentBrief` (the planning skill's text: where the block comes from, what an empty block
means, and no instruction to author one). `Scaffold` (installs it byte-equal, as it
installs the others).

- test — the installed planning skill gives no instruction to write `scenarios.md`
- test — it names the world facts covering the plan's task nodes as the source of the block
- test — it says an empty block is spec work and not a blank to fill
- test — `init` and `sync-skills` install it byte-equal to the embedded copy
