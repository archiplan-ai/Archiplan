---
affects: [AgentBrief, Scaffold]
outcome: accepted
---

# The briefing requirement outlived the briefing

`the-agent-arrives-briefed` describes the installed block as carrying `archi search` and
the installed skills. The block was cut from thirty-nine lines to nineteen and holds
neither. No test defends the sentence, so nothing failed; the requirement simply became
false and stayed green. The agent that cut the block reported it, because its contract
forbade it from editing the spec.

## Attractor

A requirement that is false and passing is worse than a missing one: it is read as the
contract, and the reader who trusts it writes against a briefing that no longer exists.
The tree has no way to notice — prose about prose is checked by nobody — so the only
mechanism is a person who happens to read both. That mechanism just worked once, by
accident, because a sub-agent was disciplined enough to report out of scope.

## Resolution

Accepted as a class, corrected as an instance. `the-agent-arrives-briefed` is amended to
describe the briefing that ships, and nothing detects the next sentence that goes stale the
same way. This one cost more than the verification case, because a false requirement is
read as the contract by the next person — and that asymmetry is recorded in
`prose-is-not-checked-against-prose`.
