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

The incident never touched the command — the branch came from raw git.
The answer is discipline, in the briefing: the mint is the only branch
maker, the base is named aloud before every fork, and new work that
builds on an unlanded unit is a poll, never the agent's own call —
continue in that unit's worktree, mint from inside it, or land it
first. Derived `a-fork-is-a-spoken-choice`.
