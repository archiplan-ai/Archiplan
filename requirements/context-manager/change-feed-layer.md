# Change feed layer

The change feed is the CM's **outbound, reactive path**, and it stays
**stateless**. It reads the Store's change-stream and **fans every change to both
sinks**; routing and authorization happen downstream.

## Reading the log

The change feed consumes a **change-stream over the outbox**, provided by the
**Store contract** ([store-layer.md](store-layer.md)) and resumable from an offset.
How the stream is implemented — LISTEN / NOTIFY, logical replication, table
polling — is the **adapter's detail**.

## Two sinks

- **Hooks.** It hands every event to the **tooling layer**
  ([tooling-layer.md](tooling-layer.md)), which routes it to the configured hooks
  and runs the action. A hook is a **server-side integration** (it enriches the
  customer's own store), so it receives every event its config routes to it — the
  config is the gate.
- **Subscriptions.** It hands every event to the **API**, which owns the socket.
  Before broadcasting to a socket, the API checks that **socket's token** with
  PermManager and pushes the events that subscriber may see
  ([api-layer.md](api-layer.md), [permission-layer.md](permission-layer.md)).

So the feed is a plain fan-out: routing lives in the tooling layer (hooks) and the
authorization gate lives in the API (subscriptions). Durable delivery state (the
per-hook offsets) lives in the tooling layer; the change feed stays stateless and
resumes from the stream on restart.

## Open questions

- **Delta granularity.** What an entry carries — a per-entity write, or something
  coarser — and how subscriptions filter the stream down to what a client asked
  for. *(Deferred — contained; it shapes the entry payload, leaving the layers as
  they are.)*
