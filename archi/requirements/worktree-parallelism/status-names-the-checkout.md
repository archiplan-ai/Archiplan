---
kind: functional
origin: intent
satisfied-by: [Cli, Seats]
deferred:
---

# Status names the checkout

One verb answers "where am I": worktree path, branch, this checkout's registry binding, the
active plan with its open wave, version state (at / dirty since), the open stress round, and
every plan on this branch with open lifecycle. It is the first verb of any working session.

## System Context

Nothing new is stored — Cli composes what it already holds: the registry entry, Archive's
current, Sessions' open round, PlanFile state. The open-plans listing is what makes handoff self-describing: a
fresh checkout of a pushed branch discovers mid-flight work without verbal instructions.

## Satisfy

`Cli.status` composes what the subsystems already hold: the checkout and branch, this
checkout's binding from `Seats`, the active plan and wave, version state, the open stress
round, and every plan with open lifecycle — the first verb of any working session.

- test — status names the seat, its plan, wave, version state and open round (`status_names_the_checkout_and_its_open_work`)
- test — gitless status names the missing repository instead of guessing (`a_gitless_project_refuses_mutation_loudly`)
