# Context Manager — architecture

The CM is a **fixed core**: a request path (External GraphQL API → Authn →
entities → Perm → Store) and a reactive outbound path (the **change feed**), with
**two ports** — TokenManager and Store Decorator. A delivery attaches a
**generated domain edge**, and **tooling** is set by config.

The **core source stays fixed** across customers: a delivery generates code into
the project and the per-customer binary is recompiled with it. "Fixed" describes
the core source; the build runs each delivery.

## Layers and ports

```
        UI (people)        CLI / MCP (agents)
            └───────┬────────┘
                    │ raw bearer / TLS   (requests in)
                    ▼
   ┌──────────────────────────────────┐
   │       External GraphQL API        │
   └──────────────────────────────────┘
                    ▼
   ┌──────────────────────────────────┐
   │              Authn                 │
   └──────────────────────────────────┘
                    ▼
   ┌──────────────────────────────────┐
   │         GraphQL entities          │
   │   Identity (fixed) │ Domain (gen) │
   │     each entity carries a Guard   │
   └──────────────────────────────────┘
                    │ Guard
                    ▼
   ┌──────────────────────────────────┐
   │  Perm · PermManager [extensible]  │
   └──────────────────────────────────┘
                    ▼
   ┌──────────────────────────────────┐
   │   Store · Store Decorator [PORT]  │
   └──────────────────────────────────┘
                    │ outbox
                    ▼
   ┌──────────────────────────────────┐        tooling config
   │            Change feed            │ ◄──────  (registry + hooks)
   │  fills & drains outbox · deltas  │
   └──────────────────────────────────┘
          │ hook (CM → tool)     │ deltas → API (subscription socket)
          ▼                      ▼
     CLI / MCP tools      API serves the socket → UI / agents (live)

   ─────────────────────────────────────────────────────────────────────
   PORTS:  TokenManager (at Authn)   ·   Store Decorator (at Store)
   PER-CUSTOMER crates: port adapters + the domain edge + the policy
   Requests flow down (in); the change feed routes deltas out — hooks to tools,
   and to the API for the subscription socket.
```

## Layers

Below is a one-line map of the layers; the detail lives in each layer's own file.

### API layer
The single contract every client speaks — one endpoint, one schema, composed at
compile time from core identity and the domain edge. → [api-layer.md](api-layer.md)

### Auth layer
Turns a raw bearer into a verified **principal** (and mints carriers at sign-in).
Uses two ports: the **TokenManager** (carrier shape — mint + parse) and the
**Store Decorator** (fetch the record by `tokenId`, verify the `secret`). →
[auth-layer.md](auth-layer.md)

### Core entities — identity
The fixed core entities (accounts, namespaces, projects, membership, access,
tokens), built through the entity utility, so each carries a Guard. →
[core-entity-layer.md](core-entity-layer.md)

### Domain entities
The customer's generated context model (the domain edge), built through the same
entity utility and compiled in alongside identity. →
[domain-entity-layer.md](domain-entity-layer.md)

### Permission layer
Authorizes every call: the Guard proxies into the policy; the baseline allow-model
ships, and granular policies plug into the same contract. →
[permission-layer.md](permission-layer.md)

### Store layer · Store Decorator [port]
Every entity reads and writes through one Store Decorator; the engine stays inside
the adapter (default Postgres). → [store-layer.md](store-layer.md)

### Change feed
Owns **all hooks** and the **delta management**. It **fills** the Store outbox on
write, then **drains** it: **hooks** to tools (per the tooling config), and the
**delta stream** to the API layer, which serves subscriptions over the socket. →
[change-feed-layer.md](change-feed-layer.md)

### Tooling layer
Inbound, tools are API clients; they also bring the **config** — the tool registry
and which events fire which hooks — that the change feed consumes. →
[tooling-layer.md](tooling-layer.md)

## Crates and build

The CM is a **Cargo workspace**: a fixed core, per-customer adapter and domain
crates, and one application binary assembled from them. How the crates depend on
each other, why the generated edge stays compatible with the fixed core, and how
the generated code is trusted are in [crates.md](crates.md).

## Request flow

A request runs down the layers: the **API** receives it, **Authn** verifies the
caller, the target **entity's Guard** asks **Perm** to authorize, and the resolver
reads or writes through the **Store**.

```
API → Authn → entity · Guard → Perm → Store
```

Each step's detail is in that layer's file.

## Open questions

- **Audit.** Whether the CM keeps an audit trail — who did what, and when — and in
  what shape. This is **cross-cutting** (it touches every mutation, the tooling
  calls, and the store), so it lives at the architecture level. Open.
