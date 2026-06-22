# Log pulling (pollers)

Logs reach the Log Analyzer two ways: **pushed** over HTTP (see
[`ingest-log-format.md`](ingest-log-format.md)) or **pulled** from the user's
existing log store. This document covers the pull side — the Log Analyzer polls
the user's store on their behalf and feeds the same pipeline the push path feeds.

## Requirement

For a service configured to pull, the Log Analyzer must read error logs from the
user's existing store on a **per-service cadence**, resume from a **durable
cursor** with no gaps and no duplicates, and reduce each fetched record to the
**same canonical entry** a pushed log produces — so that everything downstream
(dedup, analysis) is identical regardless of how a log arrived.

## The model

| Aspect | Requirement |
| --- | --- |
| **Cadence** | each service polls on its own interval; only enabled services poll |
| **Cursor** | polling resumes from a durable per-service position; the cursor advances only when the whole fetched window is persisted, so a mid-window failure re-fetches rather than skips |
| **Bounded window** | a single fetch covers a bounded time slice; a lagging cursor walks the backlog forward over successive polls rather than issuing one unbounded query |
| **Concurrency** | concurrent polls are capped, and a service is never polled twice at once |
| **Scope** | reads are read-only and credentialed; the source credential is held in the Log Analyzer's own encrypted store (see [`secrets.md`](secrets.md)) |

## Source kinds

A pulled service points at a store and a per-service filter. The supported
stores are extensible; each must adapt to the one canonical entry:

| Store | Selection |
| --- | --- |
| **Graylog** | a stream + query, intersected with a global error filter |
| **Sentry** | an org + project |
| **(future)** | other stores (CloudWatch, Loki, …) each need an adapter to the canonical entry |

## Normalization

Whatever the store, each fetched record must yield the canonical entry —
extracting at least: severity level, message body and stack, correlation/trace
id, and the build revision the failure occurred on (which grounds the analysis).
It then enters the **single ingest path** the HTTP endpoints also use, so pulled
and pushed logs are indistinguishable from there on — same deduplication, same
analysis (see [`incident-lifecycle.md`](incident-lifecycle.md)).

