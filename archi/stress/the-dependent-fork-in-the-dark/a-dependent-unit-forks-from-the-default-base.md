---
affects: [Seats]
outcome: breaking
---

# a dependent unit forks from the default base

Ask for feature B in the session that built feature A, while A stands
unlanded in its worktree. The session works from the primary checkout,
so the bound-checkout question never fires.

## Attractor

The mint forks the home branch from the primary's HEAD — main — and
says nothing about the fork point. B compiles against a world without
A. The gap surfaces days later as missing code, a broken rebase, or a
duplicate implementation.

## Resolution

Two answers, one per layer. The command: a bare `--base <branch>` names
the home fork point, and every mint report prints the branch and commit
the home branch forked from — a wrong base becomes visible in the same
breath it happens. The briefing: new work that builds on an unlanded
unit is a poll, never a silent mint — continue in that unit's worktree,
fork from its branch with `--base`, or land it first. Derived
`the-mint-names-its-fork`.
