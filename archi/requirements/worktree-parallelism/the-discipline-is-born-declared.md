---
kind: functional
origin: intent
satisfied-by: [Scaffold]
deferred:
---

# The discipline is born declared

`archi init` writes `protected = ["main"]` into every new manifest: a
fresh project starts under the seat discipline, and leaving it is the
conscious act of deleting the line — never a forgotten default. Projects
born before the default opt in with the same line.

## System Context

An opt-in nobody remembers to declare protects nobody. Scaffold owns the
manifest's birth; the line switches on mutation-needs-a-seat, and a
declared discipline without git refuses loudly rather than evaporating.

## Satisfy

`Scaffold` writes the line at init, create-only; sync never touches the
manifest.

- test — init_e2e::a_fresh_init_stands_up_a_building_project
- test — init_e2e::the_verbs_around_init_keep_their_contracts
