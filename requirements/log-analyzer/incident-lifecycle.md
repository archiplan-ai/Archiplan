# Incident deduplication & lifecycle

The Log Analyzer receives a high-volume stream of error logs in which the **same
failure recurs thousands of times**. The product requirement is to present **one
incident per genuinely-distinct failure**, kept current as that failure recurs —
not a flood of duplicates.

## Deduplication requirement

- A stream of near-identical errors must collapse to **one incident per distinct
  failure**.
- Both **exact repeats and textual variants** of the same failure must attach to
  the same incident — the variable parts of a message (ids, timestamps, values)
  must not split one failure into many incidents.
- **Distinct failures must stay distinct** — over-merging is as wrong as
  under-merging.
- Deduplication must not require reading code; it operates on the logs alone.

## The incident

One record per distinct failure. It must hold:

| Aspect | Requirement |
| --- | --- |
| **identity** | a stable identifier that survives recurrences — the same failure maps to the same incident over time |
| **evidence** | a representative error message plus the variant messages attached to it |
| **occurrence** | how many times it has happened, and when it was first and last seen |
| **state** | whether it has been analyzed yet, plus operator suppression |
| **result** | a one-line summary and the structured verdict (see [`incident-output.md`](incident-output.md)) |

## Lifecycle requirements

- **Birth.** An incident exists as soon as a failure is recognized as distinct —
  before it is analyzed. A per-incident analysis run then grounds it with a
  verdict (see [`harness.md`](harness.md)).
- **Recurrence.** A repeat of a known failure **must update the existing
  incident** (count, last-seen) and **never create a new one**.
- **Regression.** If a failure recurs after a period of dormancy (it had stopped,
  then returned), the incident must record that recurrence distinctly.
- **Suppression.** An operator must be able to suppress a failure (so its logs
  stop opening or updating an incident) and to exclude an incident from the
  context an analysis reads — without deleting it.

## Requirement

A failure that occurs N times must converge to **one** incident that stays
current — collapsing exact and variant repeats — without ever losing the
distinction between genuinely different failures.

## Questions

- **Identity stability.** As a failure's wording evolves over releases, the
  system must keep recognizing it as the same failure. How is drift (one failure
  splitting into several incidents) detected and corrected? UPD - partially handled by agent investigating git history
