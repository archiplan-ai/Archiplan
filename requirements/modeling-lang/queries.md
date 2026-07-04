# Model Query API

Queries let users request particular slices of a model. Results are rendered as a list of syntactical statements —
the same statements that would recreate the sliced part of the model, so a result can be read, diffed, or replayed
back into a spec.

## Operations

- **List ports of a node** — every statement that attaches to a port of the node:

```
> ports Orders
Payments(send_confirmation) confirm(OrderId) Orders(handle_confirmation);
Orders { handle_confirmation = ConfirmationHandler(handle_confirmation); }
```

- **Filter by view** — any query can be restricted to the edges of one or more [views](./modeling-lang.md#views).

- **Check** — model-completeness [findings](./errors.md#errors-vs-findings): edges whose shape conformance drifted
  after a classifier edge was removed, carried traffic that matches no delegation, delegated ports with no attached
  connections, views with no edges, types with no instances.
