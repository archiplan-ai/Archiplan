# Agent Interface

An interface allowing agents to interact with the full functionality of Archiplan in the most convenient
format for an LLM. The [modeling language](./modeling-lang/modeling-lang.md) is already JSON — statements
cover definitions, mutations and reads — so this interface is a thin **envelope**: a request carries a
statement batch, a response carries the structured results the language already defines (outcomes, errors,
findings, statements), plus a revision. There is no second vocabulary and no translation layer.

## Principles

- **Flexible spec editing** — the interface must not constrain the optimal design flow. Any statement the
  language accepts is accepted here, in whatever order the agent's reasoning produces it; correctness is
  guarded by atomicity, [idempotent definitions](./modeling-lang/modeling-lang.md#definitions) and the
  error contract, not by workflow.
- **One vocabulary** — requests carry the language's own statement objects; query results and cascade
  reports come back as the same statement objects, replayable as-is.
- **Stateless by construction** — the language has [no ambient
  scope](./modeling-lang/modeling-lang.md#addressing--no-ambient-scope): every statement is absolutely
  addressed, so every request is self-contained. Two requests share nothing but the model.
- **Safe retries** — inherited from the [error contract](./modeling-lang/errors.md): identical
  restatements — definitions and edges alike — are noops, batches are atomic, failures leave the model
  untouched. An agent may resubmit a whole corrected batch without diffing state first.

## Request

One request is one batch against one model. How a request names its target model (connection, id, path) is
transport-specific and out of scope here.

```json
{
  "statements": [
    { "stmt": "define", "node": "Orders.RefundHandler" },
    { "stmt": "app", "node": "Orders", "port": "events",
      "route": { "node": "RefundIssued" },
      "inner": { "node": "RefundHandler", "port": "handle" } },
    { "stmt": "check" }
  ],
  "expect_revision": 41,
  "dry_run": false
}
```

| field           | meaning                                                                                  |
|-----------------|-------------------------------------------------------------------------------------------|
| statements      | the batch: an ordered array of [statement objects](./modeling-lang/modeling-lang.md#statements) |
| expect_revision | optional optimistic-concurrency guard; the request is rejected if the model's revision differs |
| dry_run         | optional; execute and report the full results — including cascades — then roll everything back |

## Response

```json
{
  "status": "ok",
  "revision": 42,
  "results": [
    { "result": "applied" },
    { "result": "applied" },
    { "result": "findings", "findings": [] }
  ]
}
```

| field    | meaning                                                                                    |
|----------|-----------------------------------------------------------------------------------------------|
| status   | `ok` or `error`                                                                              |
| revision | the model revision after the request (see [Revision](#revision))                            |
| results  | one entry per statement, in batch order (present when `status` is `ok`)                     |
| error    | the failure, with the failing statement's `index`; the whole batch was rolled back          |

### Result objects

Tagged by outcome, mirroring the [result contract](./modeling-lang/errors.md#statement-results):

```json
{ "result": "applied" }
{ "result": "noop" }
{ "result": "applied", "cascade": [ { "stmt": "define", "node": "Orders.ConfirmationHandler" }, "..." ] }
{ "result": "graph", "nodes": [ "...node objects..." ], "edges": [ "...edge objects..." ] }
{ "result": "findings", "findings": [ "...finding objects..." ] }
```

- `cascade` appears on `delete` and node-`redefine` results: everything removed, rendered as replayable
  statement objects in creation order. With `dry_run` it is a preview of what *would* be removed.
- `graph` carries [query](./modeling-lang/queries.md) output — the requested slice as plain nodes and edges,
  with meta (types, kinds, ports, views, scope nesting encoded in path ids) preserved.

### Error object

The [error shape](./modeling-lang/errors.md#error-shape) verbatim, plus the failing statement's index. The
`subject` is the statement object as submitted; `hint` is a runnable statement an agent can feed straight
back.

```json
{
  "index": 2,
  "code": "E_SHAPE_VIOLATION",
  "message": "source Invoice does not match the source pattern of `confirm`",
  "subject": { "stmt": "conn-edge", "conn": "confirm",
               "source": { "node": "Invoice", "port": "x" },
               "carrier": "OrderId",
               "target": { "node": "Orders", "port": "handle_confirmation" } },
  "refs": [ { "kind": "node", "path": "Invoice", "id": 17 } ],
  "expected": { "anchor": "Service", "rel": "type_of" },
  "actual": "Invoice",
  "hint": { "stmt": "query", "scopes": ["Orders"] }
}
```

### Finding objects

[Findings](./modeling-lang/errors.md#errors-vs-findings) are tagged by kind, with the fields each kind
carries:

```json
{ "kind": "shape_drift", "statement": { "stmt": "conn-edge", "conn": "confirm", "...": "..." },
  "slot": "source", "expected": { "anchor": "Service", "rel": "type_of" }, "actual": "Payments" }
{ "kind": "unrouted_traffic", "statement": { "stmt": "conn-edge", "...": "..." }, "port": "Orders.events" }
{ "kind": "delegated_port_without_connections", "port": "Orders.handle_confirmation" }
{ "kind": "empty_view", "view": "fault_prop" }
{ "kind": "type_without_instances", "type_kind": "conn", "name": "confirm" }
```

Finding kinds are append-only, like error codes.

## Revision

The wrapper maintains a monotonically increasing revision per model. It increases whenever the model
changes and is untouched by noops, reads and dry runs. Granularity (per statement vs per request) is an
implementation choice; only monotonicity and change-detection are contractual.

`expect_revision` makes edits conditional: if the model moved since the agent last looked, the request is
rejected with `E_STALE_REVISION` and the agent regrounds (`query`, `check`) before retrying.
Concurrent editing semantics beyond this guard belong to [multiplayer](./multiplayer.md) and
[versioning](./versioning.md).

## Protocol errors

Failures of the envelope rather than of a statement. Same object shape as statement errors, no `index`,
codes append-only:

| code             | raised when                                                     |
|------------------|------------------------------------------------------------------|
| E_BAD_REQUEST    | the request is not valid JSON or violates this contract          |
| E_STALE_REVISION | `expect_revision` does not match the model's current revision    |

## Conventions for agents

Not enforced, but the intended way to use the interface:

- `define` what should exist; `redefine` only to replace a node's internals or a type's shape. A definition
  that contradicts the model fails loudly (E_REDECLARED) instead of silently diverging, so stale
  assumptions surface early.
- Submit a plan as one atomic batch. On error, fix the statement at `index` and resubmit the whole batch —
  statements are idempotent, so anything already applied by an earlier batch just noops.
- Preview destructive edits: `delete` or node-`redefine` with `dry_run` returns the full cascade without
  applying it.
- Reground from the model (`query`, `check`) instead of trusting a stale context window; `check`
  after substantial edits surfaces what drifted.
- A cascade report is replayable: to undo an accidental delete, submit the cascade back as a batch.
  References that eroded meanwhile surface as findings, not errors.

## Transport

The contract is transport-agnostic: one JSON request, one JSON response. An MCP tool per model is the
natural first fit; a plain HTTP endpoint serves the same contract. The [CLI](./cli.md) speaks the same
envelope with `--json`. Authentication, addressing and distribution are covered by
[saas](./distribution/saas.md) / [on-prem](./distribution/on-prem.md).
