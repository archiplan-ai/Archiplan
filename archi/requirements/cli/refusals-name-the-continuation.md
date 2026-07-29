---
kind: functional
origin: stressor(a-batch-dies-mid-round)
satisfied-by: [DocMint, Cli]
deferred:
---

# Refusals name the continuation

A mint refused because the record already stands tells the author to continue,
never just that a wall exists: re-opening the round that is open names it as this
round and points at `stress add`; re-minting an existing skeleton says the file
stands and its slots await prose. A re-run of a half-applied batch therefore
converges — applied lines answer with continuations, the unapplied tail applies.

## System Context

Batch is fail-fast by contract, so a mid-round death leaves applied lines behind;
the recovery move is re-running the same batch. Refusals written for the fresh
case (`already open — close it or fold it`, `slug taken`) prescribe the wrong
repair in the re-run case and turn convergence into guesswork
(stressor: a-batch-dies-mid-round). The distinction is cheap at the refusal site:
the verb knows whether the standing record is the very one being re-minted.

## Satisfy

`DocMint` (the same-record detection: an open round re-opened by its own slug, a
skeleton re-minted at its own path); `Cli` (the refusal text carries the
continuation).

- test — re-running a round's full batch after a mid-batch death converges: applied lines answer with continuations, the tail lands (`a_replayed_batch_converges`)
- test — `stress open` of the already-open round names it as the continuation, not a wall (`a_replayed_batch_converges`)
