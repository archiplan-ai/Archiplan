# CLI

`archi` is a thin runner for statement batches. The [modeling language](./modeling-lang/modeling-lang.md) already
covers the full operation set — definitions, mutations, reads — as JSON statements; the CLI submits batches and
renders results for humans. It adds no vocabulary of its own.

## Commands

```
archi exec [--dry-run] [--expect-revision <N>] [<batch.json> | -]
```

Executes a batch (a JSON array of [statements](./modeling-lang/modeling-lang.md#statements)) from a file or stdin,
atomically. `--dry-run` reports the full results — including delete/redefine cascades — and rolls everything back;
`--expect-revision` is the optimistic-concurrency guard from the [agent interface](./agent-interface.md#revision).

Read sugar, for not having to write JSON at the prompt — each wraps a single-statement batch:

```
archi ports <Path> [--in <View,...>]
archi check [--in <View,...>]
archi dump [--in <View,...>]
```

## Output

Human-readable by default: results and cascades rendered in the spec's
[pseudo-syntax](./modeling-lang/modeling-lang.md#notation), findings and errors as one-liners carrying their
`hint`. `--json` emits the [agent interface](./agent-interface.md) response envelope instead — one contract, two
skins.

## Exit codes

| code | meaning                                                        |
|------|-----------------------------------------------------------------|
| 0    | batch applied (or noop'd), or successful read                   |
| 1    | the batch was rejected; the structured error is printed         |
| 2    | the invocation itself is malformed (unknown verb, bad flags)    |

## Locating the model

How the CLI finds the model it operates on (workspace discovery, an explicit flag, a remote connection) is
distribution territory: [saas](./distribution/saas.md), [on-prem](./distribution/on-prem.md).
