# Context Manager — crates and build

How the CM is packaged into crates, how the pieces depend on each other, why the
generated edge stays compatible with the fixed core, and how the generated code is
trusted. For the runtime shape — layers and request flow — see
[architecture.md](architecture.md).

## Crates and dependencies

The CM is a **Cargo workspace**. Each part is a crate, in two groups.

**Fixed core crates** — the same for every customer:

- the **Store** port — the decorator contract;
- the **TokenManager** port — carrier shape: carrier ↔ `(tokenId, secret)`
  (mint / parse);
- the **allow** crate — the policy contract, the baseline policy, and the Guard;
- the **entity utility** — builds a GraphQL entity already wired with a Guard and
  bound to the Store;
- the **identity** entities;
- the **core** crate — schema assembly and the serving pipeline.

**Per-customer crates** — generated or selected at delivery:

- a **Store adapter** — the decorator implementation; default Postgres;
- a **TokenManager adapter** — the default scheme, or a customer's (e.g. a
  Keycloak JWT);
- the **domain edge** — the generated GraphQL entities (LLM-written);
- the **policy** — the baseline allow-policy from core, or a more granular
  implementation selected at delivery;
- the **application** — the binary.

**How they depend.** Each customer crate depends on **core**: the Store and
TokenManager adapters implement a core trait, and the domain edge uses the core
contracts and the entity utility. The **application binary** depends on the
customer crates — it names them, wires them together, and runs. Dependencies run
one way: **application → customer crates → core** (the application depends on
core directly as well).

core is named by the others and stays fixed. The application binary is the
per-customer assembly: it selects the concrete Store adapter, the TokenManager,
and the policy, builds the schema (core identity + the domain edge), and the
build recompiles it with the selected crates. Because the domain edge reaches the
store through the core Store contract, the application supplies whichever adapter
fits (default Postgres, or another).

```
   per-customer (generated / selected)          fixed core (trait crates)
   ┌──────────────────────────┐                 ┌───────────────────────────┐
   │  Store adapter           │ ─ implements ──► │  Store · TokenManager     │
   │  TokenManager adapter    │ ─ implements ──► │  · allow · entity utility │
   │  Domain edge (entities)  │ ─ depends on ──► │  · identity · core        │
   └──────────────────────────┘                 └───────────────────────────┘
              ▲
              │ lists & wires the customer crates
   ┌──────────────────────────┐
   │  Application (binary)     │  recompiled per customer
   └──────────────────────────┘
```

## Generate and attach — the core stays fixed

The domain edge is a **generated crate** that depends on the core trait crates.
The application lists it alongside core identity, and the binary is recompiled.
The core stays fixed because:

- the **schema is composed** from the core identity entities and the domain edge
  merged at the root, so adding entities extends the composed schema while every
  core name stays put;
- **the allow contract is generic over entity type** — a domain entity carries
  its type, grants reference that type, and the policy reads them the same way for
  every entity, so the same allow code serves new entities as they come;
- the **Store contract is uniform** — every entity reads and writes through the
  one Store port, so the same core store code serves it.

The variable part — the domain edge crate and the chosen adapters — lives in the
project; the core stays one source, built per delivery.

## Trusting the generated edges

The generated edges (domain entities, their grants, the store binding) carry the
per-customer risk, so confidence comes from **what is total or checked against an
oracle**:

- **Verification by construction.** The entity utility's typed API and the
  compile-time wiring make the valid shapes the only ones that compile — an
  entity carries a Guard, a query maps to read, a type is backed by the store.
  Everything else surfaces as a build failure.
- **Check the artifact against the model.** The generation input is the
  customer's declared model; the generated schema is verified as a **total
  transform** of it — every declared entity and field present, types matching — a
  deterministic structural diff.
- **Round-trip.** model → schema → model recovered from the schema, required to
  equal the input.
- **Invariants over the whole set**: every reachable entity has a Guard; every
  granted type exists; every entity binds to the one Store Decorator.
- **One oracle-backed fuzz** at the authorization boundary: the Guard is a small
  total function `(token rights, operation, type) → decision`, checked against
  the grant table as its oracle.
- **One human gate at delivery.** The artifact is small and read once: a reviewer
  signs off on the **schema** (the customer's model in GraphQL) and the **grant
  matrix** (account × type × R / RW) — the whole security surface.
