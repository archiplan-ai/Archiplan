# Store layer — Store Decorator [PORT]

Persistence behind **one port**. Every entity — identity and domain alike — reads
and writes through the **Store Decorator**; the engine stays hidden inside the
adapter. The default adapter is Postgres.

The Store holds the identity data — including the **token record** the auth layer
verifies against ([auth-layer.md](auth-layer.md)) — and the customer's context,
behind the one contract. It also provides the **outbox** — an append-only log the
write path appends to in the same transaction — and exposes a **change-stream**
over it (resumable from an offset) that the change feed consumes
([change-feed-layer.md](change-feed-layer.md)).

The decorator is a **swappable adapter crate** that implements the core Store
trait. The contract expresses **intent** (what to read or write); the engine
specifics live inside the adapter.

The storage contract, the default Postgres adapter, audit, backup, and scaling are
in [database.md](database.md). How the change-stream is implemented (LISTEN /
NOTIFY, logical replication, table polling) is the adapter's detail; how its events
are shaped and consumed is the change feed's concern
([change-feed-layer.md](change-feed-layer.md)).

## Open questions

- **Write retry-safety (idempotency).** How a retried write stays safe from
  duplicates — id-addressed writes (update / delete by id are naturally safe;
  create carries a client-supplied id), or a separate idempotency key. This rides
  on the store's write model, which is still open.
