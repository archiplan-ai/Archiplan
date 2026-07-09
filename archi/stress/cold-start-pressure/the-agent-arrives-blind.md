---
affects: [Agent, Scaffold]
outcome: breaking
---

# The agent arrives blind

A fresh clone, a fresh session: the agent finds `archi.toml`, greps, and starts
guessing — it hand-writes a requirement with a `status:` field the schema never
had, "fixes" a finding by editing a `closed:` stamp, and learns the tool one
`E_DOC` at a time. The strict schemas were built to end this loop, but their text
lives in archiplan's own repository, not in the project the agent is standing in.

## Attractor

The knowledge that drives archiplan well — the verbs, the lifecycle rule, the
schema shapes — travels with the archiplan repo, not with the projects it models.
Every new repository re-derives it from error messages, and an agent's context
window starts empty in exactly the place the workflow assumed it wouldn't.

## Resolution

The briefing became an init artifact: the workflow and merge skills land verbatim
under `.claude/skills/`, and `CLAUDE.md` gains a fenced block naming the source
dir, the check loop, `archi search` and the skills — the project carries its own
operating knowledge from minute one.
Answered by `the-agent-arrives-briefed`.
