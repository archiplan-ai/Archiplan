---
kind: functional
origin: stressor(the-fact-spans-two-repositories)
satisfied-by: [Gherkin, WorldDoc, Planner]
deferred:
---

# A scenario names where it runs

A scenario carries a tag naming the member whose tree runs it — `@runs:backend` — and a
scenario with no such tag runs in the project's own repository. `check` resolves the name
against the declared members and refuses an unknown one. A fact may cover nodes across
members; a single scenario runs in one place, and a walk that crosses members is written
as one scenario per member, joined by the fact that carries them.

## System Context

`the-grammar-takes-the-whole-language` bought tags, so the declaration costs no new
syntax. Multi-repo work already resolves member by member — tasks declare outputs per
member, link refs are member-qualified — and a scenario with no home was the one artifact
in that picture with nowhere to execute. Splitting a cross-member walk into one scenario
per member loses nothing the wing needs: the fact is what holds them together, and the
fact was always the unit. What it prevents is the most convincing unverified artifact
possible — a scenario that parses, prints at a close, latches, and no runner ever sees.

## Satisfy

`Gherkin` (the tag is ordinary Gherkin and parses as one). `WorldDoc` (the scenario
carries it; absence means the project's own repository). `Planner` (the block groups by
member at the close, so an operator verifying end to end knows which tree to stand in).

- test — `@runs:backend` on a scenario resolves against the declared members
- test — a member name that no declaration carries raises a located error
- test — a scenario with no tag resolves to the project's own repository
- test — the closing block groups scenarios by the member that runs them
- test — a fact holding two scenarios with different members passes `check`
