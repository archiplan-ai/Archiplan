---
affects: [DocMint, Cli]
outcome: breaking
---

# A batch dies mid-round

A round's batch — open, six adds — dies on line four. The natural agent move,
re-running the whole batch, hits `round … is already open — close it or fold it`
on line one: fail-fast kills the re-run before the unapplied tail, and the refusal
prescribes exactly the wrong repairs.

## Attractor

Partial state plus dead-end refusals: the agent either abandons the round, hand-
edits the batch by guesswork, or — worst — obeys the message and closes a
half-minted round.

## Resolution

The refusals must name the continuation, not just the wall: re-opening the round
that is already open says "this is it — continue with `stress add`"; re-minting an
existing skeleton says "already minted — the file stands, fill its slots". Derived
`refusals-name-the-continuation`.
