---
node: Scaffold
owns: [the-agent-arrives-briefed]
---

# t2 — Scaffold

archi-implement.md gains the cleanup step between the waves and the
scenarios: when `plan next` prints the cleanup block, dispatch ONE
sub-agent with the sweep contract — the unit's whole delta
(`git diff <base>..HEAD` in the worktree), fold duplicated mechanisms
only, zero behavior change, the whole suite green with no assertion
edits, journal-anchored symbols survive as thin wrappers, an empty
sweep returns one line and nothing is committed. Then commit the sweep
(when it changed anything), run `archi link verify`, repin what the
fold drifted, and run `plan next` for the scenarios. The step never
reaches outside the unit's delta.

## Spec

- `Scaffold`
- `Function type_of Scaffold`
- `Cli.drive consult(->Command, <-Report) Scaffold.stand_up`

## Inputs

- from t1 — the cleanup block wording the briefing tells the agent to expect

## Outputs

- skills/archi-implement.md

## Stack

- include_str! embeds — rebuild, then archi sync-skills
- init_e2e byte-equality

## Verifications

### the-agent-arrives-briefed

- test — cargo test init_e2e: the installed briefing stays byte-equal to the binary's embedded copies
- manual — the step sits between the waves and the scenarios, scopes to the unit delta, and allows the empty sweep
