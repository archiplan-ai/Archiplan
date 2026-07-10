---
kind: non-functional
origin: intent
satisfied-by: [Engine]
deferred:
---

# Failure is machine-readable

Every statement in a batch yields exactly one of applied, noop or error; a batch is
atomic — any error rolls the whole batch back and reports the failing statement's
index — and reads yield their payload in place. An error is a structured object: a stable
code from an append-only catalog, a human one-liner, the offending statement as
submitted, the paths involved, the violated constraint where one applies, and a hint
phrased as a runnable next statement. Codes never change meaning; new ones may appear.
Errors reject statements; states that are legal mid-construction but suspect — unwired
ports, empty views, types without instances — are findings, surfaced by reads and never
by rejection.

## System Context

Agents author most edits and repair from what the tool tells them: a rejection they
cannot parse, an index they cannot trust after a partial apply, or a code that shifts
meaning between releases would break the repair loop the whole workflow leans on. The
read envelope carries these same objects verbatim (`agents-read-lowered-statements`), and
`findings-never-block` carries the reject/advise split up to the CLI's exit codes.

## Satisfy

`Engine` (tri-state statement results, atomic rollback with the failing index, the
structured error object, the append-only catalog).

- test — errors::batches_are_atomic, errors::a_failed_statement_leaves_no_partial_state
- test — errors::a_parse_error_reports_its_index_and_rolls_back
- test — errors::envelope_statement_error_carries_index, errors::envelope_protocol_errors
- test — semantics::outcome_json_shapes_match_the_spec
