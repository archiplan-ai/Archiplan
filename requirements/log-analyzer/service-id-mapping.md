# Service identity & id mapping

How a "service" is identified at the ingest boundary and across the system.
This is the join key for everything: a misrouted id pools logs under the wrong
service, and the agent then analyzes them against the wrong code.

## The ingest mapping problem

At the ingest boundary the producer must supply the **internal id** (the
`serviceId` field). That id is obtained by an operator after creating the
service, then hard-coded into the producer's push config.

That couples the producer to an internally-assigned identifier:

- it isn't stable across environments (the same logical service has different
  ids in staging and prod);
- it isn't meaningful to the producer (no relation to the service's own name /
  deploy identity);
- a wrong id silently pools logs under another service — there is no
  producer-side way to catch it.

Unknown or **disabled** ids are dropped at ingest (the request still 200s for
the rest of the batch), so a bad id fails silently rather than loudly.

## Requirement

A service must be addressable at ingest by a **stable, producer-meaningful key**
carried in the ingest payload — never an internally-assigned id.

The Log Analyzer **owns the service registry** — the key → service mapping and
that service's stored source configuration (see [`sources.md`](sources.md),
[`database.md`](database.md)). Ingest **validates the carried key against the
local registry** and rejects or quarantines an unknown or disabled key at the
door, so a bad key fails loudly rather than pooling logs under the wrong service.
Analysis then reads the service's stored sources by that key — no per-run
resolution against another component.
