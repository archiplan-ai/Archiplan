---
affects: [Seats.Landing]
outcome: accepted
---

# the receiving branch is stale after the merge

The pull request merged an hour ago on the forge. The local receiving
branch knows nothing until someone pulls, so every proof of integration
answers "not yet".

## Attractor

Either archi starts fetching behind the operator's back, or seats stand
for days after their work landed and the operator stops trusting the
sweep.

## Resolution

Living with it: archi never reaches the network on its own, and the
sweep proves against the local receiving branch alone. The listing says
what it compared against, so "not integrated" reads as "not here yet,
pull first" instead of a verdict about the forge. The skill, which may
use `gh`, is where a forge question belongs.
