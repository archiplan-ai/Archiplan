---
kind: functional
origin: intent
satisfied-by: [Scaffold, AgentBrief]
deferred:
---

# The briefing says what help does not

The `CLAUDE.md` block carries only what an agent cannot get by running `archi --help` or by
reading the skills it was given. It states the rule to use the tool, what this tree is, the
check loop, that delegated spec work returns as files, and that a world fact carries no noun
of the model. It lists no command syntax and no skill inventory.

## System Context

The block grew one bullet per feature and lost nothing, so it became a second copy of the
help text plus a directory of the skills — thirty-nine lines where fifteen carry meaning. A
briefing that repeats what one command prints teaches an agent to skim it, and the lines that
only live there go past with the rest.

The test is discoverability: `archi --help` prints every verb and its flags, and the harness
lists the installed skills to the agent already. What neither of them says is that designing
in chat is forbidden, that lifecycle moves only through commands, that a fan-out returns paths
and not payloads, and that a fact speaking the model is a requirement in costume. Those are the
block.

## Satisfy

`Scaffold` (the block text it installs and refreshes). `AgentBrief` (the durable carrier).

- test — the installed block lists no command flags
- test — the installed block names no skill by file path
- test — the block states the no-model-nouns rule
- test — the block states that spec work returns as files
- test — the block is shorter than twenty lines
