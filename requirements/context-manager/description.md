# Context Manager

## What it is

The authoritative layer for the customer's **context** — **the customer's
model**, its links to their **external artifacts**, and their freshness. The CM
is the **single backend**: clients (the UI for humans, CLI / MCP tooling for
agents) call its API, and the CM authenticates and authorizes every call itself.
The artifacts themselves stay in the customer's own systems; the CM holds
**references** to them and serves their current freshness. Clients can also
**subscribe** to changes and receive them live.

## Our approach: a fixed core, generated edges

The CM is a small **fixed core** plus **edges generated per customer**.

- **Fixed core** — what must be uniform and correct, built once and the same for
  every customer: identity, tenancy and access (PermManager); the **Store
  Decorator** contract; the **token-carrier contract** (TokenManager port); the
  GraphQL
  contract and runtime; the **change feed** (outbox → hooks and deltas).
- **Generated edges** — everything that can be tailored to a customer, produced
  at delivery (or reused off-the-shelf when one fits): the **database connector
  (decorator)** for their engine; the **token-carrier adapter** (the default
  scheme, or the customer's issuer); and the **domain GraphQL layer** — the
  entities for their context model, each typed into the allow-model. (The grant
  rows — who gets which type — are deployment data in the store.)

**Why generate.** Code generation is cheap now (LLM-assisted). So we **generate
everything that can be tailored** to the customer and keep fixed only what must
be uniform. The core gives correctness and consistency; the generated edges give
fit. Reuse a ready edge when it fits, generate when a new one is needed.

## Architecture

The CM's shape — layers, ports, the entity utility, the permission contract,
tooling and hooks, and how the generated edges are trusted — is in
[architecture.md](architecture.md).

## Delivery

The system is delivered **per customer**: we operate the deployment (cloud) or
the customer self-hosts it. A delivery assembles the generated edges onto the
fixed core. The full process is in [delivery.md](../delivery.md).

## Source of truth

| Entity | Authoritative owner |
| --- | --- |
| The model and its links | **Context Manager** |
| The external artifacts | The customer's own systems; the CM holds references and freshness markers to them |

## Freshness

The CM **stores the context** and serves **data sufficient to judge whether it is
fresh**: each link carries a **freshness marker** of its target, and the CM hands
that marker out so a reader can draw the fresh-or-stale conclusion. The CM also
**fires hooks** (CM → tool on an event).

Re-reading the customer's systems and updating the markers happens **outside the
CM** — by tooling. What triggers it is up to that tooling: a hook, or its own
schedule (e.g. cron). The CM stores the marker as data, so any artifact kind is
supported.
