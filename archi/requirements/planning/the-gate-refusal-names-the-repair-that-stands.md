---
kind: functional
origin: intent
satisfied-by: [Planner, AgentBrief]
deferred:
---

# The gate refusal names the repair that stands

A refusal names the repair that answers **that** refusal, and says so when a neighbouring
verb does not. The wave's file gate is answered by a declaration —
`archi plan task <id> link add` — and its refusal states in as many words that
`archi link add` does not answer it, because that gate reads the declaration files and not
the journal. The advisory checklist beside it is answered by `archi link add`, and it says
so. And a refusal that can be answered two ways names both: the file gate's second repair
is the boundary — a changed file that is not code leaves the scans through
`[audit] exclude` in `archi.toml`, the same boundary the audit and capture already share —
and the refusal says it, because the reader stuck on a lockfile or a generated artifact
cannot be expected to know a manifest key the message never names. No refusal names a
candidate list or `link confirm`.

## System Context

The refusal is read at the one moment a person is stuck, so a continuation that does not
exist costs more than silence would. One survived three units past the mechanism it
described: capture stopped proposing candidates, and the wording that sent the reader to
`archi link ls --evidence` and `archi link confirm` stayed.

The trap the second time was subtler and is the reason this claim is written this way. The
repair was rewritten to `archi link add`, and one unit later the gate changed again and the
repair became a declaration — while the test still passed, because the new refusal carries
the words `archi link add` inside the sentence that says they will not work. A test that
matches a command name matches the sentence denying it, so the check has to read what the
refusal tells the reader to run, not which words appear in it.

## Satisfy

`Planner` (both refusals of `plan next`, the advisory checklist, and the prose that
documents them).

- test — the file gate's refusal names `archi plan task <id> link add` with its three flags
- test — the same refusal says `archi link add` does not answer it, and the two statements
  are not the same line
- test — the advisory checklist names `archi link add` with the ref and the anchor it takes
- test — no refusal and no checklist names `link ls --evidence` or `link confirm`
- test — the file gate's refusal names the boundary repair: a non-code file leaves through
  `[audit] exclude`, and the sentence names the manifest key
- test — the briefing's failure modes carry the same case for the wave gate, beside the
  audit's prose-files entry
