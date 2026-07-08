---
affects: [Links]
outcome: breaking
---

# Hub wave floods review

A task on a hub node carries the node's whole derived surface, and the wave that fixes one verb
also lands a test file: one changed function, a handful of harness helpers, a few test
functions. Capture pairs every changed symbol with every spec_ref of the owning task, no content
consulted.

## Attractor

Replayed from `issues/capture-mints-a-blind-cross-product.md` as observed closing plan
`close-without-minting`: t1 on `Cli` carried 11 derived refs, the delta held ~15 symbols, and
capture minted 164 candidates — l0094–l0257 — of which 158 were retired unread. Test-file
plumbing (`MODEL`, `NEXT`, `ok`, `run`, `temp_project`) × 11 refs is noise by construction: no
reviewer will ever confirm `Agent.drive … Cli.nkp ← version_e2e.rs#temp_project`. The
confirm-or-retire review is the ratchet's quality gate, and a wall that size trains the operator
to mass-retire — the six load-bearing pairs were indistinguishable except by prior knowledge.

## Resolution

Broke, as filed. Answered this round by putting a signal between symbol and ref before minting:
a candidate is minted only when the ref's surface terms — node path, edge endpoints, payload
types, split on case and underscores — overlap the changed item's symbol path or canonical body
tokens (file-level items add their path terms). Pairs with no shared term are suppressed:
counted in the human render, listed whole under `--json`, and never subtracted — a hand
`link add` mints them asserted any time. Touches, decays, retire-subtraction and leftovers keep
their semantics. Derived: candidates-carry-signal.
