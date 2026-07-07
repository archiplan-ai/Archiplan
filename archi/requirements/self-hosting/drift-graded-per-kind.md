---
kind: functional
origin: intent
satisfied-by: [Links]
deferred:
---

# Drift graded per kind

A code-link watches the hash its kind claims: a literal link claims the exact body, an indirect
link claims the symbol's role — its interface. Formatting and comment churn move neither;
internals churn moves only literal links. The alarm has to mean something, or re-pinning becomes
a ritual that launders real drift.

## System Context

Symbols are hashed over a canonicalized token stream, not source bytes; rustfmt runs freely on
this repository.

## Satisfy

`Links` recomputes projections on verify and grades each link Clean, Drifted, Moved or Missing
against the watched hash — body for literal, interface for indirect — with evidence links never
failing the gate.

- test — reformat a linked fn body: both kinds verify Clean; rewrite its internals: literal drifts, indirect holds
- test — delete a linked item: verify grades Missing and exits non-zero for the asserted link
