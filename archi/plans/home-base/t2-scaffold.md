---
node: Scaffold
owns: [the-agent-arrives-briefed, a-fork-is-a-spoken-choice]
---

# t2 — Scaffold

Three additions, in the briefing's voice. archi.md, the mint step:
the mint is the only branch maker — never `git checkout -b`, in the
home repo or in a member; name the base aloud before the mint (the new
branch grows from the checkout the mint runs from — say which); new
work that builds on an unlanded unit is one poll question, three
options — continue in that unit's worktree, mint from inside it (the
fork then grows from its branch), or land the unit first.
archi-implement.md, Step 0:
before the first wave the orchestrator confirms the ground through the
poll tool whenever it is ambiguous — stay in this worktree and its
member worktrees from `archi status`, or attach another seat; branches
come only from the mint cascade, and a task that depends on unlanded
work standing elsewhere is a question to the user, never the agent's
own call. archi-implement.md, Sub-agents: every prompt forbids branch
creation and branch switching — sub-agents write code on the branches
the worktrees already stand on. Rebuild embeds, sync, init_e2e green.

## Spec

- `Scaffold`
- `Function type_of Scaffold`
- `Cli.drive consult(->Command, <-Report) Scaffold.stand_up`

## Inputs

## Outputs

- skills/archi.md
- skills/archi-implement.md

## Stack

- include_str! embeds in scaffold.rs — rebuild, then archi sync-skills
- init_e2e byte-equality — the gate that the copies match

## Verifications

### a-fork-is-a-spoken-choice

- manual — the mint step forbids raw branches and names the base aloud; dependent work walks through the three-option poll; implement's Step 0 confirms the ground; the sub-agent contract forbids branch creation and switching

### the-agent-arrives-briefed

- test — cargo test init_e2e: the installed briefing stays byte-equal to the binary's embedded copies
