# Link to all sources

During analysis the agent doesn't reason over the log alone — it **fetches the
service's own sources on demand** to ground the verdict. Each service links to a
set of sources; the agent reads them read-only at run time and cites what it
used.

## What a source is

A source is **anything the agent may need to consult to explain a failure** —
the service's code, its data stores, its documentation, the systems it talks to.
The set is **open-ended**: the Log Analyzer must not assume a fixed catalogue of
source types. A new kind of source is added by teaching the system how to reach
it, not by changing what an incident or a verdict is.

Each linked source carries enough to reach it: where it lives, how the agent
connects, and the credential to use (held encrypted — see [`secrets.md`](secrets.md)).

## How the link works

- Sources are configured **per service** and shareable across services without
  copies (one source backing many services).
- The agent fetches **on demand inside the per-incident run** — it is not handed
  a pre-loaded dump; it decides which sources to touch based on the failure.
- Every fetch goes through **one uniform fetch surface** that is:
  - **read-only** — a source is never written through this path;
  - **scoped** — bounded by per-source allowlists, size/row caps, and timeouts;
  - **audited** — which source, what was asked, what came back, and the run that
    issued it.
- Credentials are resolved **per source, at fetch time**, never held by the agent.
- What the agent actually used surfaces in the verdict as **cited sources** (see
  [`incident-output.md`](incident-output.md)).

The Log Analyzer **stores the source set per service** in its own registry —
each source's location, how to reach it, and its **encrypted credential** (see
[`database.md`](database.md), [`secrets.md`](secrets.md)). It **owns** this
configuration rather than resolving it from another component at run time, so a
service is keyed to its sources by the stable key carried in the log (see
[`service-id-mapping.md`](service-id-mapping.md)).

## Requirement

For a configured service the agent must be able to reach **every** source the
failure could require — of **any kind** — through one uniform, read-only, scoped,
audited fetch surface, resolving credentials per source at fetch time.
