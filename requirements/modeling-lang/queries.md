# Model Query API

Queries let users request particular slices of a model. They are read **statements** of the
[language](./modeling-lang.md#language-api) and may appear anywhere in a batch; their output comes back in the
per-statement results. Results are lists of **statements** — the same statement objects that would recreate the
sliced part of the model, so a result can be read, diffed, or replayed back into a spec. (Examples below use the
spec's [pseudo-syntax](./modeling-lang.md#notation); on the wire both queries and results are JSON statement
objects.)

## Operations

- **List ports of a node** — every statement that attaches to a port of the node:

```
ports Orders;
→ Payments(send_confirmation) confirm(OrderId) Orders(handle_confirmation);
→ Orders.handle_confirmation = ConfirmationHandler(handle_confirmation);
```

- **Dump** — the whole model, or a view slice of it, as replayable statements in creation order: `dump;` /
  `dump in data_flow;`. An unfiltered dump replays into an empty model and recreates it (creation statements render
  as `define`s, which are idempotent, so replays and retries are safe); a view slice carries the minimal
  declarations its edges need to parse.

- **Filter by view** — any query can be restricted to the edges of one or more
  [views](./modeling-lang.md#views) with `in`.

- **Check** — model-completeness [findings](./errors.md#errors-vs-findings): edges whose shape conformance drifted
  after a classifier edge was removed, carried traffic that matches no delegation, delegated ports with no attached
  connections, views with no edges, types with no instances: `check;` / `check in data_flow;`.
