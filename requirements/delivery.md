# Delivery

**Status: in progress.** How the system is delivered and deployed for a
customer.

## The deployable unit

One system, deployed **per customer**. A deployment is a customer's own instance:

- the **Context Manager (CM)** — the single backend (authn / authz / PermManager,
  the GraphQL API);
- its **store** — one database behind a decorator (default Postgres);
- the **interface (UI)** — served to the customer's users.

**Tooling (CLI / MCP)** is distributed to the customer's users and agents; it
runs on their machines and connects to that CM as a client. The **License
Server (LS)** is the vendor's central service that every deployment connects out
to (licensing + telemetry).

## Two modes — the difference is who operates it

The system is identical in both modes; only operation differs.

| | Cloud (we operate) | Self-host (customer operates) |
| --- | --- | --- |
| CM + store + UI run | on vendor infrastructure | inside the customer's contour |
| The store | our default Postgres | the customer's own database (default Postgres, or another engine via a decorator) |
| DB connection secret | ours | held in the CM's env |
| Backup, migrations, upgrades | run by us | run by the customer |
| `CM → LS` (licensing + telemetry) | internal to the vendor | egress from the contour to the LS |

## Delivery process

Delivering to a customer is **assembling the system from parts** — each part
either generated for them or reused off-the-shelf. The CM is the same binary
everywhere; a delivery just feeds it the customer's assembly.

### 1. Gather requirements

Which **database**, the customer's **context model** (the entity types in a
project), which **tooling** they need, and **who gets what access**.

### 2. Connector

Generate or reuse a **decorator** for the customer's database. The default
Postgres decorator ships ready; another engine gets a decorator built for it.
The CM talks to whatever connector through the one uniform interface
([`context-manager/database.md`](context-manager/database.md)).

### 3. Domain GraphQL layer + perms

Generate or reuse the **GraphQL entities** for the customer's context model,
together with the **permission layer** — the guard plus the
`(account, project, entity type) → R / RW` grants
([`context-manager/permission-layer.md`](context-manager/permission-layer.md)). A standard
model can use an off-the-shelf layer; a bespoke one is generated (LLM-assisted)
and validated. The GraphQL contract — queries, mutations, subscriptions — is
then live ([`context-manager/tooling-layer.md`](context-manager/tooling-layer.md)).

### 4. Register tooling

Register the **CLI / MCP tools** the customer needs, with their hooks, in the
tool config ([`context-manager/tooling-layer.md`](context-manager/tooling-layer.md)).

That is the delivery. The running CM loads this assembly — connector + domain
GraphQL + perms + tooling — and serves it. "Generate or reuse" applies to every
part: a previous customer's connector or domain layer is reused when it fits,
generated when it does not.

### Evolving a live deployment

When the customer's context model changes, regenerate the GraphQL entities while
keeping the addressing (and so the grants) stable; roll out CM / UI upgrades.
**Backup and migrations** are run by the store's owner — us for cloud, the
customer for self-host. The **CM keeps its heartbeat with the LS** for licensing
and telemetry — the single egress, the same in both modes.

## Open questions

- **Air-gapped self-host** — offline activation and lease issuance when the
  deployment reaches the LS rarely or never.
- **Where tooling / agents run** relative to the deployment, and how they are
  distributed and updated.
- **Re-provisioning** when the customer's context model changes — regenerate the
  GraphQL entities while keeping grants stable.
