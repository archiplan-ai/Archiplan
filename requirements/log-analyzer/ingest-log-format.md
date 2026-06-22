# HTTP ingest — log JSON

How a producer pushes logs into the Log Analyzer, bypassing the poller. The
producer owns severity; the analyzer does not filter — HTTP push trusts the
client and accepts everything it is given.

## Endpoints

| Endpoint | Body | Use |
| --- | --- | --- |
| `POST /ingest/logs` | a **JSON array** of entries (a batch) | bulk push |
| `POST /ingest/logs/stream` | **NDJSON** — one entry per line | streamed push |

Auth: `Authorization: Bearer <INGEST_TOKEN>` (a single shared token; empty
token disables the feature). Limits: ≤ 10 000 entries per batch, ≤ 25 MB body.

## Entry

One entry = one log occurrence. Extra keys are tolerated and ignored, so a
producer can forward a richer object without breaking ingest.

| Field | Required | Notes |
| --- | --- | --- |
| `serviceId` | **yes** | the **stable, producer-meaningful key** of the service this log belongs to — validated at ingest against the Log Analyzer's local service registry. See [`service-id-mapping.md`](service-id-mapping.md). |
| `message` | **yes** | the raw human log line — the analyzer's primary evidence. |
| `level` | **yes** | severity word; normalized but **not** filtered. |
| `occurred_at` | no | event time. Unix seconds (ms auto-detected ≥ 1e12) or ISO-8601 / `YYYY-MM-DD HH:MM:SS`; naive = UTC. Null → server `now()`. |
| `environment` | no | e.g. `prod` / `staging`. |
| `trace_id` | no | correlation id; normalized, used to link related logs. |
| `commit_sha` | no | the build SHA the log came from — pins analysis to a known revision when present. |

```json
[
  {
    "serviceId": "balances-api-node-32",
    "message": "io.lettuce.core.RedisCommandTimeoutException: Command timed out …",
    "level": "ERROR",
    "occurred_at": "2026-06-06T11:34:30.974Z",
    "trace_id": "abc-123",
    "commit_sha": "45a427a"
  }
]
```

## What happens to an accepted entry

Ingest authenticates the push and **validates the `serviceId` key against the
local service registry** — an unknown or disabled key is rejected/quarantined at
the door (see [`service-id-mapping.md`](service-id-mapping.md)). An accepted entry
is buffered; from there it is indistinguishable from a pulled log — same
deduplication, same analysis (see [`incident-lifecycle.md`](incident-lifecycle.md)).

## Open questions

- Per-service ingest tokens vs one global `INGEST_TOKEN` (rotation, blast
  radius, attributing a push to a producer).
- Backpressure / quota per service when a producer floods the pipeline.
