---
node: Scaffold
owns: [the-agent-arrives-briefed]
---

# t2 — Scaffold

Three additions to the skills, then rebuild embeds and sync. archi.md
opening: before the first `archi check` of a session, run `archi
check-update` — one line; an available update is relayed to the user,
never installed unasked. archi.md multi-repo/opening: right after a
cascade mint, run `git log --oneline <base>..HEAD` in every member
worktree — a fresh seat must show nothing; anything else is relayed
verbatim before any work starts. The same opening names the survey:
the map's rot now rides `archi check` findings — read them, they are
the worklist. archi-implement.md: the DONE anchor step gains one line —
after a squashed PR lands, the anchor is what keeps baselines on real
branches.

## Spec

- `Scaffold`
- `Function type_of Scaffold`
- `Cli.drive consult(->Command, <-Report) Scaffold.stand_up`

## Inputs

- from t1 — the survey findings the briefing tells the agent to read

## Outputs

- skills/archi.md
- skills/archi-implement.md

## Stack

- include_str! embeds in scaffold.rs — rebuild, then archi sync-skills
- init_e2e byte-equality — the gate that the copies match

## Verifications

### the-agent-arrives-briefed

- test — cargo test init_e2e: the installed briefing stays byte-equal to the binary's embedded copies
- manual — the opening runs check-update before the first check; the post-mint member sweep and the survey-findings line sit in the opening; implement's anchor step names the squash
