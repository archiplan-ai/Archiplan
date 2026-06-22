# Agent execution (the harness)

Every piece of reasoning the Log Analyzer does — deduplication and per-incident
analysis — runs as an **agent run**: a bounded, sandboxed, recorded execution of
an LLM agent against a service's sources. This document states what a run must
guarantee, independent of which agent runtime provides it.

All agent runs go through **one execution path**, so the guarantees below hold
uniformly.

## What a run must guarantee

| Requirement | Statement |
| --- | --- |
| **Single chokepoint** | every agent run — deduplication and analysis — executes through one path, so isolation, recording, and limits apply uniformly |
| **Grounded workspace** | a run that reads code does so in a working copy of the service's repository **at the incident's build revision**, not at an arbitrary HEAD |
| **Read-only** | a run cannot write, patch, fetch the network, or run arbitrary commands; it has an **explicit allowlist** of read-only capabilities and nothing more |
| **Scoped source access** | sources attached to a run are exposed read-only and per-tool allowlisted; a run can reach only the sources its service grants (see [`sources.md`](sources.md)) and never sees a raw secret (see [`secrets.md`](secrets.md)) |
| **Bounded & cancellable** | every run has a hard wall-clock limit and can be cancelled by an operator; all in-flight runs drain on shutdown |
| **Configurable model** | the model is a setting, adjustable at runtime |

## Recording a run

A run is the **audit unit** of the system. Each run must be recorded with enough
to reconstruct and account for it:

- its **anchor** — the incident it grounded, or the service whose logs it deduplicated;
- the **inputs** it received (the assembled prompt) and the **revision** it ran
  against;
- the full **agent output** (the event stream), retained for the transcript;
- the extracted **verdict** (see [`incident-output.md`](incident-output.md));
- **telemetry** — duration, outcome, and the run's **cost and token usage**,
  which feed the analytics surfaces.

## Run kinds

| Kind | Purpose | Stance |
| --- | --- | --- |
| **deduplication** | collapse incoming logs into incidents, without reading code | read-only, no verdict |
| **per-incident analysis** | ground an incident — fetch sources, write the verdict | read-only |

Source fetching happens **inside** the per-incident run, not in a separate
pass — the analysis agent decides which sources to read based on the failure.

## Requirement

All agent reasoning must run through one execution path that sandboxes the run to
the right code revision, enforces a read-only capability allowlist, bounds and
cancels the run, and records it as a costed, anchored audit unit.

## Open questions

- **Credential ownership.** Source secrets are held by the Log Analyzer itself
  (see [`secrets.md`](secrets.md)); the model credential is a secret like any
  other — where should it live (env, the same encrypted store, a KMS)?
- **Cross-tool run identity.** A run is the audit unit; when its verdict flows to
  the Context Manager / Task Tracker ([`incident-output.md`](incident-output.md)),
  does the run identity travel with it, or only the incident?
- **Per-run cost ceiling.** A run is bounded by a wall-clock timeout and a
  global daily run cap; a per-run token/cost budget is not yet a requirement.
