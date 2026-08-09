---
kind: functional
origin: stressor(a-closed-row-still-licenses-mutation)
satisfied-by: [Seats.Registry, Seats.Guard]
deferred:
---

# only an active row binds

Every lookup that grants something reads active rows alone: the
mutation guard, the verdict gate, the plan owner and the member
resolution of a cascade. A closed row answers nothing — it is history,
and history licenses no writes.

## System Context

Rows used to vanish when the work ended, so "the row exists" and "the
work is live" were one fact. Once a row outlives its work the two part,
and every reader that trusted the old equality must say which one it
means.

## Satisfy

`Seats.Registry` answers binding lookups from active rows only, so
`Seats.Guard` refuses in a checkout whose only row is closed, with the
recipe it prints for any unbound tree.

- test — a closed row leaves its checkout unbound and does not own its
  plan
