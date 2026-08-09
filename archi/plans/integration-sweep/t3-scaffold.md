---
node: Scaffold
owns: [the-agent-arrives-briefed]
---

# t3 — Scaffold

Three skills carry the old story. archi.md: the registry moves by `ls`
and `close`, rows are closed and never deleted, the opening looks for
an active row, and a seat that waits on a pull request is normal — it
frees itself once the work is in the receiving branch. archi.md failure
modes: a seat still standing after a merged pull request means the
local receiving branch has not been pulled. archi-finish-worktree.md:
the sideways landing keeps the worktree until the pull request lands —
push, open the PR, and let the next archi command free the folder; the
abandon step is `archi worktree close <slug>`; where `gh` is available
the skill may ask the forge whether the PR merged and then close.
archi.md also gains the answer to "what was done here": ask the
registry and the spec — `worktree ls`, `plan list`, `version list` —
before any archaeology in git. Rebuild embeds, sync, hold init_e2e.

## Spec

- `Scaffold`
- `Function type_of Scaffold`
- `Cli.drive consult(->Command, <-Report) Scaffold.stand_up`

## Inputs

- from t2 — the shipped verb name, the state words and the ls filter the briefing quotes

## Outputs

- skills/archi.md
- skills/archi-finish-worktree.md

## Stack

- include_str! embeds in scaffold.rs — rebuild, then archi sync-skills
- init_e2e byte-equality

## Verifications

### the-agent-arrives-briefed

- test — cargo test init_e2e: the installed briefing stays byte-equal to the binary's embedded copies
- manual — no skill names `worktree drop`; the waiting seat is described; the "what was done here" answer names the registry and the spec first
