# Database (Log Analyzer DB)

The Log Analyzer owns its **own** database — a **PostgreSQL**, hybrid relational
+ JSONB store — separate from the Context Manager's authoritative Sys DB.

## Why a database here at all

The Log Analyzer is not a stateless pass-through. Between a log arriving and an
operator reading a grounded verdict, the system must **buffer** a high-churn
stream, **remember** which failures it has already seen so recurrences merge
instead of duplicating, **queue** agent work and drain it in a correctness-
preserving order, and **account** for every agent run it spends money on. None
of that survives in memory across restarts, and all of it must be correct under
concurrent agents. That is what the DB is for.

## Why its own DB, separate from the Context Manager

The two stores hold different *kinds* of data with different lifecycles. The Sys
DB is the slow-moving **authority** — the spec and its links, read and written
deliberately. The Log Analyzer's DB is an **operational** store: a fast-moving
log buffer, a work queue under constant claim contention, and an ever-growing
run-history ledger. Coupling that churn to the authoritative store would put
hot-path throughput and unbounded audit growth on the system's correctness
spine. The Log Analyzer integrates with the Context Manager by **reference** (see
[`incident-output.md`](incident-output.md)), not by sharing its tables.

## What the DB must serve

1. **A high-churn ingest buffer.** Pushed and pulled logs land continuously and
   are drained in bulk by deduplication — write-heavy, transient data that is
   buffered, consumed, and discarded (see [`incident-lifecycle.md`](incident-lifecycle.md)).
2. **The deduplicated truth.** The set of distinct incidents and their occurrence
   history — long-lived, and what recurrences merge into. Deduplication must be
   able to look up existing incidents by their identity and recent history
   cheaply.
3. **Transactional work queuing.** Agent work is claimed by competing workers
   under leases, with a strict serial ordering on the organic analysis path.
   Atomic claim — no job taken twice, none lost on a crash — is a **correctness**
   requirement, not an optimization: it underwrites the dedup guarantee (a later
   analysis must see an earlier sibling's incident so duplicates merge).
4. **An accountable run ledger.** Every agent run is recorded append-only with
   its transcript and its cost/token telemetry, anchored to what it was about.
   This is both the audit trail and the source of the analytics surfaces (spend,
   tokens), so it must support cheap aggregation over time windows (see
   [`harness.md`](harness.md)).
5. **The per-service source registry, with secrets at rest.** Which sources each
   service links to (keyed by the service's stable key), shareable across
   services, including **encrypted** credentials never stored or logged in
   plaintext (see [`sources.md`](sources.md), [`secrets.md`](secrets.md)).
6. **Operator-tunable configuration.** A small set of governing knobs an operator
   can change at runtime and have read back without a redeploy.
7. **Modest volume, self-hostable.** Throughput is per-service log rates and a
   capped daily run count, not web scale. Correctness under concurrency decides
   the engine; the product must also self-host on commodity infrastructure.

The Log Analyzer **stores** each service's source configuration and credentials
in this DB (encrypted at rest); it **owns** them rather than resolving them from
another component at analysis time (see [`sources.md`](sources.md),
[`service-id-mapping.md`](service-id-mapping.md)).

## Why PostgreSQL

The data has **two natures**, and Postgres holds both in one ACID transaction:

| Data | Nature | Representation |
| --- | --- | --- |
| Buffered logs | Transient, write-heavy buffer | relational rows, bulk-discarded |
| Incidents + occurrence history | Hybrid — the verdict is a document, the stats and keys are relational | JSONB verdict + relational columns |
| Work queue | Relational, concurrency-critical | rows claimed under transactional locks + leases |
| Run ledger | Append-only audit + numeric telemetry | tables with time-windowed aggregation |
| Source registry + credentials | Relational config + encrypted blobs | tables, sharing relations, encrypted columns |
| Runtime settings | Small key/value | a table read with a short cache |

Requirement → mechanism: transactional, lock-based claim for the queue (3);
JSONB and relational rows in one transaction for incidents (2); plain
append-only tables with indexed time windows for the ledger (4); encrypted-at-rest
columns for the source credentials (5); a single modest instance for all of it
(7). Postgres is also the path of least resistance for self-hosting — on every
enterprise approved-vendor list, identical managed or on-prem.
