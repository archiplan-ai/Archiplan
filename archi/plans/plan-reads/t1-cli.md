---
node: Cli
owns: [a-plan-reads-by-name]
---

# t1 — Cli

Give `plan show` an optional name: with it, load that plan directly (record or
legacy) and render — never touching `.current`; without it, the active plan as
today. An unknown name lists the plans. The router keeps `show` free.

## Spec

- `Cli`
- `Service type_of Cli`
- `Agent.drive invoke(->Command, <-Report) Cli.build`
- `Agent.drive invoke(->Command, <-Report) Cli.check`
- `Agent.drive invoke(->Command, <-Report) Cli.incidence`
- `Agent.drive invoke(->Command, <-Report) Cli.init`
- `Agent.drive invoke(->Command, <-Report) Cli.link`
- `Agent.drive invoke(->Command, <-Report) Cli.nkp`
- `Agent.drive invoke(->Command, <-Report) Cli.plan`
- `Agent.drive invoke(->Command, <-Report) Cli.query`
- `Agent.drive invoke(->Command, <-Report) Cli.read`
- `Agent.drive invoke(->Command, <-Report) Cli.repo`
- `Agent.drive invoke(->Command, <-Report) Cli.req`
- `Agent.drive invoke(->Command, <-Report) Cli.search`
- `Agent.drive invoke(->Command, <-Report) Cli.session`
- `Agent.drive invoke(->Command, <-Report) Cli.status`
- `Agent.drive invoke(->Command, <-Report) Cli.stress`
- `Agent.drive invoke(->Command, <-Report) Cli.version`
- `Agent.drive invoke(->Command, <-Report) Cli.worktree`

## Inputs

## Outputs

- crates/archi/src/main.rs
- crates/archi/src/plans/mod.rs
- crates/archi/tests/plan_e2e.rs

## Stack

## Verifications

### a-plan-reads-by-name

- cargo test plan_e2e — a named show renders on an unbound checkout and no .current appears
- cargo test plan_e2e — nameless show serves the active plan; an unknown name lists the plans
