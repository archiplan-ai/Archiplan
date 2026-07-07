---
affects: [Links]
outcome: breaking
---

# One blob of links

A drift storm lands after a hard refactor: the audit reports decayed evidence, dark deltas and a
canonicalizer mismatch in one sweep — and every finding attributes to the same model element,
because the whole traceability machinery is one node.

## Attractor

Triage is impossible at the model's granularity: journal truth, hash contract, grading and wave
capture are distinct failure domains wearing one name, so the operator distrusts all of them at
once and retires links wholesale — traceability dies of coarse attribution.

## Resolution

Bends: the recovered model was too coarse exactly where pressure concentrates. Answered this
round by articulating the subsystem — `Links.Journal` (the append-only truth), `Links.Canonizer`
(the versioned hash contract), `Links.Grader` (projection and audit) and `Links.Capture` (wave
deltas) become children with their own ports and internal flows, so each failure domain is its
own element. Derived: link-truth-is-append-only, hash-contract-is-versioned.
