# Storage

**Status: in progress.** One store behind a **decorator**. The CM talks to it
through a uniform interface; the engine is hidden. Default PostgreSQL.

## One store, two parts of the schema

| Part | Holds | Shape |
| --- | --- | --- |
| **Fixed (core identity)** | accounts, namespaces, projects, membership, project access, tokens — see [`core-data-model.md`](core-data-model.md) | the same for every customer — we define it; it does not vary |
| **Context (per-customer)** | the customer's context data | generated per delivery, shaped for the customer ([`../delivery.md`](../delivery.md)) |

This document describes the **store and the core identity entities**. The
context part is dynamic — its shape is the customer's, generated at delivery —
so it is not modelled here.

Both parts live in the **same** store, behind the **same** decorator, so an
operation can span them — a project and its context go together, a query over
identity and context resolves in one place. How strongly (FKs, one transaction)
depends on the engine; the default Postgres gives it fully.

**A deployment serves one customer.** The store holds that one customer's data;
namespaces group the customer's teams and work. Whether **we operate the
deployment** (default Postgres, run by us) or the **customer self-hosts** it
(their own database, connection secret in the CM's env) is an operational
choice, not an architectural one — the system is the same either way.

## The decorator — one interface over the whole store

The CM talks to the store through a **single uniform interface** — for identity
**and** context alike — and never to an engine directly. **The CM never knows
the engine**: query syntax, indexes, change-feed transport — none of it leaks
up; how a decorator implements the interface is its own business, and swapping
the decorator changes nothing for the CM.

**The contract is engine-agnostic.** It expresses **intent** (what to read or
write), never engine specifics. All richness lives **inside** the decorator. An
engine qualifies as a decorator only if it can satisfy the whole contract.

**Decorators are added per integration:** the default Postgres decorator, and
another engine when a deployment needs it.

- The decorator is **chosen by config**. Default: the **Postgres decorator**
  (its engine features hidden underneath).
- For the **SQL family**, SQLx's `Any` (`AnyConnection` / `AnyPool`) lets one
  decorator serve Postgres / MySQL / SQLite / MariaDB by URL scheme — one code
  path, already in the stack. (SeaORM is the heavier ORM-shaped alternative.)
- **Non-SQL engines** — a bespoke decorator implementing the same interface,
  only if a customer requires it.

The model is **one interface + per-engine decorators**; sqlx covers the SQL
family under one path.

## What any decorator must provide — the contract

Engine-agnostic, stated as intent. How a decorator delivers each is its own
business:

- safe concurrent writes;
- consistency on write — a write and its effects land together; references stay
  valid;
- structured queries (filter / lookup) over identity and structured data;
- store and retrieve the per-customer context (document-shaped);
- an ordered, replayable change feed captured with the writes that caused it.

Modest volume.

## Postgres — the default decorator

Postgres is the **default** — the engine we run for what we host. It is the
default because it satisfies the contract cleanly and passes enterprise review;
it is **not imposed** — a customer's own engine satisfies the same contract via
its decorator, with whatever guarantees that engine provides.

The default Postgres decorator implements the contract as:

| Data | Nature | Representation |
| --- | --- | --- |
| Core identity + structured data | Relational | Tables, FKs, indexes |
| Per-customer context | Document | JSONB |
| Change feed | Stream | Transactional outbox; at-least-once, ordered delivery with retry |

ACID gives consistency-on-write in one transaction across both parts. Enterprise
fit: on every approved-vendor list, identical managed (RDS / Aurora / Cloud SQL
/ Azure) and on-prem, mature backup / HA / monitoring / compliance, DBAs already
on staff.

## Audit, backup, migrations

- **Audit lives in the store** — the same DB, alongside the data it records.
- **Backup and migrations are configured at the client level** — whoever owns
  the store runs them (us for the default Postgres we host; the customer for a
  self-hosted DB). The system does not run them centrally.

## Why not the others — as the default engine

| Candidate | Why not the default |
| --- | --- |
| SQLite / libSQL | Single-writer; fine as a small decorator target, not the default authority. |
| Document stores (MongoDB) | Forfeit referential integrity and cheap relational queries. |
| Graph DBs (Neo4j, Kuzu, …) | Access pattern is filters + 1–2 hops, not deep traversal; a separate engine to operate. |
| MySQL / MariaDB | Viable decorator target, but weaker JSON indexing — Postgres stays the default. |
| CockroachDB | Distributed-consensus tax for scale we don't have; revisit only for multi-region HA. |

## Scaling

Growth paths — read replicas, sharding by namespace / project — all within
Postgres. The enabling invariant: transactions are never wider than one
namespace, and the vast majority not wider than one project.
