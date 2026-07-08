---
affects: [Search, SourceTree]
outcome: surviving
---

# The KB outgrows the scan

A real project runs archi for two years: forty intents, twelve hundred requirement
files, ninety stress rounds, a model of two thousand elements. Every `archi search`
re-reads and re-scores all of it. The no-index answer that kept the truth single now
bills its price on every keystroke of an agent loop that searches ten times a minute.

## Attractor

Scan-on-query trades staleness for latency, and latency compounds in agent loops.
The trap is answering it early: an index bought now (with its staleness class, its
invalidation hooks, its second store) to serve a corpus that fits in one filesystem
pass with room to spare — the classic premature optimization wearing a scalability
costume.

## Resolution

Holds. The scan is one pass, linear in corpus bytes: parse, tokenize, score, rank —
no quadratic joins, no per-query recompile of anything but the model the verb needs
anyway. The imagined two-year corpus is a few megabytes of markdown; a filesystem
walk plus lexical scoring over it lands well under interactive latency on commodity
disks, and the compile of the model dominates it. If a corpus someday outgrows one
pass, the remedy is a cache keyed by file mtime — an optimization invisible in the
contract, adoptable then without touching this round's requirements. Measured on
this repository at implementation time and recorded in the plan's scenarios.
