# Log Analyzer

## What it is

A tool in the agent harness that turns a stream of production **error logs**
into a small set of **analyzed incidents**, each carrying an agent-written
root-cause verdict grounded in the service's own sources (code, data, docs), and
groups related incidents into **failure episodes** with a fault-tree explanation
of what to fix.

It **owns its own database** (the incoming-log buffer, incidents, agent runs,
the work queue, and the **per-service source registry** — each service's sources
and their encrypted credentials) and runs the analysis agents. It integrates
with the rest of the system by **reference**: it pulls/receives the user's logs
but never owns the upstream store, and it reads each service's own source systems
read-only at analysis time.

## Flow

1. **Ingest.** Logs arrive — pushed over HTTP (`POST /ingest/logs`, see
   [`ingest-log-format.md`](ingest-log-format.md)) or pulled from the user's log
   store (Graylog / Sentry, see [`log-pulling.md`](log-pulling.md)) — and are
   accepted, keyed to a service, and queued for processing.
2. **Deduplication.** Incoming logs are collapsed to **one incident per distinct
   failure**: exact and variant repeats attach to an existing incident; a
   genuinely-new failure opens a new, not-yet-analyzed incident. This operates on
   the logs alone — no code is read. See [`incident-lifecycle.md`](incident-lifecycle.md).
3. **Per-incident analysis.** Each new incident gets its **own** agent run with
   a context scoped to that one failure (see
   [`context-isolation.md`](context-isolation.md)). The agent fetches the
   service's configured sources on demand — reads the repo at the incident's
   build revision, queries data stores, reads docs — and writes the verdict: root
   cause, classification, severity, suggested fix, and the sources it cited. See
   [`sources.md`](sources.md). Every agent run — deduplication and analysis — is
   spawned through the same agent harness; see [`harness.md`](harness.md).
4. **Episode synthesis (analytics).** Incidents of one real-world failure event
   are grouped into an **episode** and synthesized into a **fault tree** — the
   symptom decomposed to its root causes, with the minimal set of fixes. See
   [`episodes.md`](episodes.md), [`fault-trees.md`](fault-trees.md).
5. **Output.** The episode (blast radius + cut-set fixes) is the primary **Task
   Tracker** material; toward the **Context Manager** the Log Analyzer reports
   only a spec→code link desync/annotation, never an incident. See
   [`incident-output.md`](incident-output.md).

## Owns vs references

| | |
| --- | --- |
| **Owns** | its DB — the incoming-log buffer, the incidents, the agent-run ledger, the work queue, and the **per-service source registry** (sources + encrypted credentials). See [`database.md`](database.md). |
| **References (does not own)** | the user's **Logs** (pulled/received, never the upstream store); each service's **source systems** themselves (code / data / docs — read at analysis time, read-only); the **Context Manager** (read for its dependency graph; the sink for spec→code link signals — see [`incident-output.md`](incident-output.md)). |

## Talking to the Context Manager

The Log Analyzer relates to the Context Manager two ways:

- **As an API client.** It **reads the Context Manager's dependency graph** to
  orient fault trees (see [`fault-trees.md`](fault-trees.md)), and **reports a
  spec→code link desync** back to it (see [`incident-output.md`](incident-output.md)).
  It does **not** fetch sources or credentials from the Context Manager — those
  are stored locally (see [`secrets.md`](secrets.md), [`database.md`](database.md)).
- **As a registered tool (hook).** It exposes a CLI/MCP handle, registered in the
  Context Manager's tool registry like the other tools, so the Context Manager
  can invoke it via hooks. Which CM events fire the Log Analyzer is not yet
  defined.

## Requirements in this folder

- [`ingest-log-format.md`](ingest-log-format.md) — the JSON a producer POSTs to the HTTP ingest endpoint.
- [`log-pulling.md`](log-pulling.md) — the poller pull side (Graylog / Sentry) feeding the same pool.
- [`service-id-mapping.md`](service-id-mapping.md) — how a service is identified in ingest and across the system.
- [`incident-lifecycle.md`](incident-lifecycle.md) — deduplication and the incident's recurrence lifecycle.
- [`episodes.md`](episodes.md) — grouping related incidents into one failure event.
- [`fault-trees.md`](fault-trees.md) — per-episode fault tree, cut sets, and fix priority.
- [`verdict-quality.md`](verdict-quality.md) — confidence calibration and the operator feedback loop.
- [`sources.md`](sources.md) — the per-service link to all sources the agent fetches.
- [`secrets.md`](secrets.md) — how the agent obtains source credentials (from the Context Manager).
- [`incident-output.md`](incident-output.md) — the finished-incident structure: the incident to the Task Tracker, a link signal to the Context Manager.
- [`context-isolation.md`](context-isolation.md) — one analysis run per incident, with an isolated context.
- [`harness.md`](harness.md) — the agent harness and how a run executes.
- [`database.md`](database.md) — the Log Analyzer's own DB and its tables.
