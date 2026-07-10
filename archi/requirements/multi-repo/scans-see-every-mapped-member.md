---
kind: functional
origin: intent
satisfied-by: [Links, Members]
deferred:
---

# Scans see every mapped member

The wave-open index, capture, verify's candidate search and the audit walk every
mapped member, key their items by member-qualified path, and say per member what
they scanned and what they could not reach — an absent checkout narrows the scan
and is reported, never silently shrinking coverage. The exclusion boundary stays
one setting: a bare pattern applies in every member, a qualified pattern scopes to
one.

## System Context

Capture is git-free by construction — it diffs tree snapshots — so scanning more
trees is a widening, not a redesign. What must not happen is silent narrowing:
a scan that skips an unmapped member without saying so reads as clean coverage
when it is blindness.

## Satisfy

`Links` (`Capture` scans the mapped set and keys the index by qualified path; `Grader` audits each
member against its own baseline and tags findings by member), `Members` (one resolution consulted
by every scan).

- test — the wave-open index holds qualified keys for two mapped members
- test — the audit reports per member: findings under one baseline, a recovery note where none exists, an unreachable note for an absent checkout
- test — a bare exclude pattern applies in every member and a qualified one in exactly its member
