---
affects: [Scaffold, Agent]
outcome: surviving
---

# The installed skill drifts

Two releases later the binary embeds a sharper workflow skill, but every
initialized repository still briefs its agents with the copy init installed — or
the team tuned their installed copy, and a refreshing re-run would flatten the
tuning. Either direction, the installed briefing and the binary's disagree.

## Attractor

The briefing is data shipped as files: the moment it lands it has two owners — the
binary embedding tomorrow's copy and the repository that committed today's. Any
automatic reconciliation picks a winner silently, and both possible winners are
sometimes wrong.

## Resolution

Holds as designed: reconciliation is never automatic. Create-only `init` never
flattens the tree on a reflexive re-run, so a re-run cannot cost a morning's work.
The refresh is a separate, deliberately-invoked verb — `archi sync-skills` — that
overwrites the installed briefing (the skills and the fenced CLAUDE.md block) with
the binary's copies: a match is `ok`, an absent file is restored, any divergent one
is overwritten. It touches only the briefing, never the model. Invoking the verb
*is* the operator choosing the binary as the winner — nothing picks behind their
back, and a team that has tuned its copy simply does not run it.
