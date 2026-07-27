---
kind: functional
origin: intent
satisfied-by: [Scaffold, Seats]
deferred:
---

# Skills open by switching

Every workflow skill — spec, plan, implement — opens the same way: status, then find the
seat. Work that continues a unit switches into the worktree already carrying it — one seat
carries spec, plan and code, and implementing a spec continues in the spec's own worktree.
Only work nothing carries mints a seat through the CLI and switches into the printed path.
Only then does the skill's own flow begin.

## System Context

The opening is a skill step because only the agent can change its own directory; everything
else — the lookup, the mint, the instruction — is Cli (worktrees-mint-on-demand,
context-follows-the-checkout). Scaffold ships the step at the head of every workflow skill it
installs (AgentBrief).

## Satisfy

`Scaffold` ships the opening step at the head of every workflow skill it installs: status,
find the seat, switch into it, mint only work nothing carries; `Seats` answers the lookup
and the mint. Each skill also polices its own freshness — `sync-skills` first, re-read on
`updated`.

- test — the installed skills carry the opening protocol verbatim from the binary (`the_briefing_lands_verbatim_and_the_fence_appends_once`)
