---
affects: [Seats]
outcome: breaking
---

# an explicit base drifts off the audited line

Mint with `--base <member>=<branch>` where the named branch does not
contain the pinned version's recorded baseline — a typo, a squashed
history, or plain haste.

## Attractor

The seat quietly continues a line the archive never audited. The gap
surfaces days later, at the landing or in a foreign diff, with nothing
in the mint's output to point back to the moment of choice.

## Resolution

The escape stays open — a refusal here would relock the squash case
the gate just learned to respect — but the choice becomes visible: when
a baseline is recorded and the named base does not contain it, the mint
prints one note naming the member, the branch and the baseline. Derived
`an-off-baseline-base-says-so`.
