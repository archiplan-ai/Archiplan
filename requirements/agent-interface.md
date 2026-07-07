# Agent Interface

An interface allowing agents to **read** an Archiplan model in the most convenient format for an LLM. The
[modeling language](./modeling-lang/modeling-lang.md) is already JSON — its read statements (`query`, `check`)
return the structured graph and findings the language defines — so this interface is a thin **envelope**: a
request carries a batch of read statements, a response carries their results. There is no second vocabulary and no
translation layer.

Editing is out of scope here by construction: a model is stored as [`.arch` source](./modeling-lang/source-format.md),
the only source of truth, and changes by text edit and recompile. The statement layer has no mutation vocabulary
(no rename, delete, untag or redefine), and this envelope accepts only reads — a write statement is rejected as a
protocol error. Agents that need to change a model edit the source files and run [`archi check`](./cli.md); this
interface is how they ground themselves before and after.

## Principles

- **Read anything the language can read** — any read statement the language accepts is accepted here, in whatever
  order the agent's reasoning produces it. The [subgraph query](./modeling-lang/queries.md) composes filters
  (types, kinds, views, scopes); `check` reports [findings](./modeling-lang/errors.md#errors-vs-findings).
- **One vocabulary** — query results come back as the language's own node and edge objects, and cascade-free:
  every path is absolute, so a result is self-describing and its statements are replayable as source.
- **Stateless by construction** — the language has [no ambient
  scope](./modeling-lang/modeling-lang.md#addressing--no-ambient-scope): every statement is absolutely addressed,
  so every request is self-contained. Two requests share nothing but the model.
- **Safe by construction** — reads never change the model, so retries, replays and concurrent requests are always
  safe; there is no revision to guard and no partial state to roll back.

## Request

One request is one batch of read statements against one model. How a request names its target model (connection,
id, path) is transport-specific and out of scope here.

```json
{
  "statements": [
    { "stmt": "query", "scopes": ["Orders"], "kinds": ["connection"] },
    { "stmt": "check" }
  ]
}
```

| field      | meaning                                                                                          |
|------------|--------------------------------------------------------------------------------------------------|
| statements | the batch: an ordered array of read [statement objects](./modeling-lang/modeling-lang.md#statements) (`query`, `check`) |

A statement that is not a read is a [protocol error](#protocol-errors): the model is edited as source, not through
this envelope.

## Response

```json
{
  "status": "ok",
  "results": [
    { "result": "graph", "nodes": [], "edges": [] },
    { "result": "findings", "findings": [] }
  ]
}
```

| field    | meaning                                                                                    |
|----------|-----------------------------------------------------------------------------------------------|
| status   | `ok` or `error`                                                                              |
| results  | one entry per statement, in batch order (present when `status` is `ok`)                     |
| error    | the failure, with the failing statement's `index`; the whole batch was rolled back          |

### Result objects

Tagged by outcome, mirroring the [result contract](./modeling-lang/errors.md#statement-results). A read batch
yields graphs and findings:

```json
{ "result": "graph", "nodes": [ "...node objects..." ], "edges": [ "...edge objects..." ] }
{ "result": "findings", "findings": [ "...finding objects..." ] }
```

- `graph` carries [query](./modeling-lang/queries.md) output — the requested slice as plain nodes and edges, with
  meta (types, kinds, ports, views, scope nesting encoded in path ids) preserved.
- `findings` carries `check` output.

### Error object

The [error shape](./modeling-lang/errors.md#error-shape) verbatim, plus the failing statement's index. The
`subject` is the statement object as submitted; `hint` is a runnable statement an agent can feed straight back.

```json
{
  "index": 0,
  "code": "E_UNKNOWN_NAME",
  "message": "unknown view `data_flow`",
  "subject": { "stmt": "query", "views": ["data_flow"] },
  "refs": [ { "kind": "view", "path": "data_flow" } ],
  "hint": { "stmt": "query" }
}
```

### Finding objects

[Findings](./modeling-lang/errors.md#errors-vs-findings) are tagged by kind, with the fields each kind carries:

```json
{ "kind": "unrouted_traffic", "statement": { "stmt": "conn-edge", "...": "..." }, "port": "Orders.events" }
{ "kind": "unused_port", "port": "Orders.handle_confirmation" }
{ "kind": "empty_view", "view": "fault_prop" }
{ "kind": "type_without_instances", "type_kind": "conn", "name": "confirm" }
```

Finding kinds are append-only, like error codes.

## Protocol errors

Failures of the envelope rather than of a statement. Same object shape as statement errors, no `index`, codes
append-only:

| code          | raised when                                                                             |
|---------------|-----------------------------------------------------------------------------------------|
| E_BAD_REQUEST | the request is not valid JSON, violates this contract, or carries a non-read statement (the interface is read-only) |

## Conventions for agents

Not enforced, but the intended way to use the interface:

- **Read to ground, edit in source.** Reground from the model (`query`, `check`) instead of trusting a stale
  context window; then make changes by editing the [`.arch` source](./modeling-lang/source-format.md) and
  recompiling. `check` after an edit surfaces what the change left incomplete.
- A query result is replayable source: every node and edge renders as an absolute-path statement, so a slice can
  be pasted into a module and recompiled.
- On a compile error, fix the offending `file:line:col` the diagnostic names and recompile — statements are
  idempotent, so unchanged source re-applies as no-ops.
- Scope a query before reading the whole model: `scopes`, `types`, `kinds` and `views` compose to the slice that
  answers the question, which is cheaper to reason over than the full graph.

## Transport

The contract is transport-agnostic: one JSON request, one JSON response. An MCP tool per model is the natural first
fit; a plain HTTP endpoint serves the same contract. The [CLI](./cli.md) speaks the same read contract with
`--json`. Authentication, addressing and distribution are covered by [saas](./distribution/saas.md) /
[on-prem](./distribution/on-prem.md).
