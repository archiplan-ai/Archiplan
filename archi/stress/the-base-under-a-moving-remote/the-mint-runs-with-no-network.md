---
affects: [Seats.Mint]
outcome: breaking
---

# the mint runs with no network

Mint on a plane, behind a proxy that eats the remote, in a repository
with no remote at all, or against a host that hangs for thirty seconds.

## Attractor

A verb that reaches the network becomes a verb that fails when the
network does. The operator learns that minting is unreliable and starts
creating branches by hand — the exact habit the briefing forbids.

## Resolution

The refresh is best-effort and never a gate: a failed or absent fetch
leaves the mint working from the local ref, and the report says which
it used and why. `--no-fetch` skips the attempt outright. Derived
`the-refresh-never-blocks-the-mint`.
