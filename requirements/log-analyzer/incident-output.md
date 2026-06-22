# Finished-incident output → Task Tracker (+ a link signal to the Context Manager)

Once an incident is analyzed, its result leaves the Log Analyzer two ways, to two
different recipients:

- the **incident itself** goes to the **Task Tracker** as task material;
- toward the **Context Manager** the Log Analyzer reports **only a spec→code link
  desync/annotation** — never an incident, because the Context Manager accepts
  only context mutations (links, freshness, spec, id mappings), not findings.

This document specifies what the incident carries internally and what crosses
each boundary.

## What the Log Analyzer produces internally

An analyzed incident = an **envelope** (the incident identity + occurrence stats)
carrying a **verdict** (the agent's grounded analysis).

### Envelope

| Field | Notes |
| --- | --- |
| `id` | incident identifier |
| `service` | service name |
| `identity` | a stable key for the failure, the same across recurrences |
| `level` | max severity observed |
| `summary` | one-line impact header |
| representative message + variants | the representative error plus the variant messages attached to it |
| `count`, `first_seen_at`, `last_seen_at` | occurrence stats |
| `status` | analyzed |

### Verdict (the agent's result)

| Field | Notes |
| --- | --- |
| `classification` | 1–3 labels: `code_bug` · `bad_input` · `bad_data` · `no_data` · `config` · `infra` · `transient` · `unknown` |
| `severity` | `low` / `medium` / `high` — operator urgency |
| `confidence` | `low` / `medium` / `high` |
| `root_cause` | 1–3 sentences citing a concrete `file:line` |
| `suggested_fix` | optional diff / 1–2 sentences, or null |
| `affected_files` / cited sources | the code (and data/doc) locations the agent read and cited |

Duplicates are not a verdict field — a repeat failure is **merged** into the
incident (count bump + its variant messages), not pointed at by id (see
[`incident-lifecycle.md`](incident-lifecycle.md)).

```json
{
  "incident_id": 462,
  "service": "balances-api",
  "identity": "…",
  "summary": "Unguarded read of an absent cache key crashes the price lookup",
  "verdict": {
    "classification": ["code_bug"],
    "severity": "medium",
    "confidence": "high",
    "root_cause": "the price lookup reads an absent cache key without guarding the empty case; the deployed build predates the guard that already exists upstream",
    "suggested_fix": "redeploy the current build — the guard is already merged",
    "affected_files": ["<path to the implicated source file:line>"]
  }
}
```

## What crosses each boundary

The Context Manager is a context **provider**, not a findings sink — its only
writes are link / freshness / spec / id-mapping mutations. So the result splits
by recipient:

- **To the Task Tracker — task material.** `suggested_fix` + `classification` +
  `severity` (with the summary and the stable identity) become a task: a
  `code_bug` with a concrete fix is a candidate task. When the incident is part
  of an **episode** ([`episodes.md`](episodes.md)), the **episode** is the unit
  sent — one task for the whole failure event, carrying its blast radius and the
  minimal **cut-set fixes** ([`fault-trees.md`](fault-trees.md)), with the
  incidents as its leaves — rather than one task per incident. The Task Tracker
  is the **sink for the finished incident or episode**; idempotency and
  recurrence apply to episodes as they do to incidents.
- **To the Context Manager — only a link signal.** The verdict's cited
  `affected_files` resolve to the spec node(s) whose linked code they implicate;
  the Log Analyzer reports a **desync / annotation on that spec→code link**
  through the Context Manager's read-write *mutate-context* path — the same path
  used to report freshness/desync. It **never sends an incident**.

## Open questions

- **Task trigger.** What creates a Task Tracker task — every `code_bug`? operator
  confirmation? a severity threshold? (The Task Tracker itself is not yet
  specified.)
- **Identity.** The id the Task Tracker references — the internal `incident_id`,
  or the stable `(service, failure-identity)` key that survives re-analysis?
- **Dedup across tools.** When the same incident recurs, the existing Task
  Tracker item must be updated (recurrence bump), not duplicated — the Log
  Analyzer already merges recurrences internally, so the outbound contract
  carries the recurrence rather than re-emitting.
- **Link resolution to CM.** How an `affected_file` resolves to a Context Manager
  spec node — i.e. which spec→code link the desync/annotation attaches to.
