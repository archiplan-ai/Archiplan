# Tooling layer

Tooling is how agents reach the CM — the mirror of the UI for humans. It works in
**two directions**: a tool **calls the CM** as a client, and the CM **calls the
tool** through a hook.

## A tool as a client (inbound)

A tool — a CLI or an MCP service driven by an agent — calls the CM's API over TLS,
exactly as the UI does. It **proxies the user's bearer**: it carries the user's
token and acts with exactly that token's grants. (A token may be scoped to a subset
of the account's grants when it is created — [auth-layer.md](auth-layer.md).)

Every such call:

- carries the user's bearer over TLS;
- is **authenticated and authorized** by the CM (PermManager —
  [permission-layer.md](permission-layer.md));
- carries the **tool's secret**; the Auth layer verifies it **through the tooling
  layer** to learn which registered tool sent it — separate from the bearer, which
  carries the account's grants;
- speaks an **explicitly versioned contract** — an unsupported version is rejected.

## The config (tool registry)

A single **central YAML config** lists **all** the tools the CM knows. Each tool
has a **`kind`** — **`push`** (the CM invokes it via hooks) or **`pull`** (it only
calls the CM / subscribes) — a **`label`**, and a **`secret`** (kept as a hash). A
**`push`** tool also has its **`type`** (`cli` | `mcp`), a **`health`** handle, and
a list of **`hooks`** — each a **handle** to `invoke` (its kind is the tool's
`type`), the **events** (`on`) that fire it, and a **`note`**:

```yaml
tools:
  - label: archi
    kind: push                  # push | pull
    type: mcp                   # cli | mcp
    secret: "<hash>"            # shared key, kept hashed, set at registration
    health: ping                # handle for the startup reachability check
    hooks:
      - invoke: on_link_changed # the handle
        on: [link.changed]
        note: keep archi's link graph in sync
      - invoke: reindex
        on: [model.updated]
        note: rebuild the local index

  - label: laptop-agent
    kind: pull                  # only calls the CM / subscribes
    secret: "<hash>"
```

The `secret` **stays inside the tooling layer** (kept as a hash); see Authenticity
below. The tooling layer exposes the list of **connected tools** (`{label}`) for
the UI.

## Startup checks

On boot the tooling layer validates the config and probes the **`push`** tools:

- **Reachability.** It calls each push tool's **`health`** handle to confirm it is
  reachable. A `pull` tool reaches the CM on its own, so the CM probes only push
  tools.
- **Known entities.** It checks that the **entity types** named in the hooks' `on`
  events exist in the store. The tooling layer has **store access** — it keeps its
  per-hook offsets there ([change-feed-layer.md](change-feed-layer.md)) — so it
  reads the schema to verify them.

## Hooks (outbound)

A **hook is the CM invoking a tool** on an event — one-way (CM → tool): the CM
calls, the tool reacts (updates its own store, runs a job). A hook implies the tool
keeps state that must stay in sync, so delivery is **reliable**: the event sits in
the outbox in the same transaction as the change, and it is delivered
**at-least-once, ordered, and cursor-resumable** — a tool that was unavailable is
invoked for every event it missed. The change feed streams the events
([change-feed-layer.md](change-feed-layer.md)); the tooling layer keeps the
**per-hook offset**, runs the action, and advances on the tool's **ack**.

**Execution — two transports:**

- **CLI** — the CM runs the configured command **locally** (a tool packaged with
  the deployment): the event goes in, `exit 0` is the ack.
- **MCP** — the CM is an **MCP client** calling the tool's MCP handle: the
  tool-call result is the ack. A remote tool rides TLS / the MCP transport.

**Authenticity.** On an inbound call the tool presents its `secret`; the **Auth
layer verifies it through the tooling layer**, which holds the **hash** — the
secret stays inside the tooling layer, and the answer is **which registered tool**.
The `secret` answers **which tool**; the user **bearer** answers **which account,
and what it may do** — the two are separate. The other direction — the **tool
trusting that a hook is from the CM** — rides the transport: **TLS** to a known CM,
or the MCP connection's own auth. A **CLI** hook runs locally, so its identity is
the host's.

**Reachability.** A **`push`** tool is one the CM can reach — a local CLI, a
sidecar, a routable MCP server — and drives via hooks. A **`pull`** tool reaches
the CM on its own (e.g. an agent on a laptop): it **subscribes** and pulls.
