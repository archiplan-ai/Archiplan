---
affects: [Incidence]
outcome: breaking
---

# Under-stressed wall

A stress round closes and the incidence report auto-fires — the designed moment of maximum
attention. The reader gets the matrix, one or two findings that matter, and then one `[info]
under-stressed` line for every term no stressor touched.

## Attractor

Reproduced three times on this very repository: `first-pressure` buried two alerts and a warn
under 31 info lines out of 39 columns; `adoption-pressure` printed 42; `order-pressure` 41. The
majority are pure data vocabulary — `Tokens`, `Ast`, `Report`, `Command` — terms no stressor
would ever sensibly press. The actionable tail (which *behavioral* components went unpressed) is
indistinguishable from the wall, the reader learns to skip the block wholesale, and the one
finding that must stay loud goes quiet. NKP already solved this for its slice with a default
class filter (`Data type_of _` dropped, behavior kept); the under-stressed sweep has no
equivalent.

## Resolution

Broke, as filed. Answered this round by scoping the under-stressed sweep to behavioral terms:
zero columns classified under `Data` (the `type_of` closure, NKP's own boundary) emit no finding
by default; `--all-terms` widens the sweep back for audits that want the vocabulary too. The
matrix and every other finding keep seeing all columns — the filter lives at the emission site,
nowhere else. Derived: under-stressed-names-behavior.
