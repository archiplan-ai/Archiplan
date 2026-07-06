# CLI

`archi` is a thin runner over the [modeling language](./modeling-lang/modeling-lang.md): it executes statement
batches, compiles [`.arch` source projects](./modeling-lang/source-format.md), and renders results for humans. It
adds no vocabulary of its own.

## Commands

```
archi exec [--dry-run] [--expect-revision <N>] [--model <file>] [--preset <file>] [--json] [<batch.json> | -]
```

Executes a batch (a JSON array of [statements](./modeling-lang/modeling-lang.md#statements)) from a file or stdin,
atomically, against a statement-log model file (default `archi.json`). `--dry-run` reports the full results —
including delete/redefine cascades — and rolls everything back; `--expect-revision` is the
optimistic-concurrency guard from the [agent interface](./agent-interface.md#revision). Reads
([queries](./modeling-lang/queries.md) and `check`) are statements like any other and run through `exec`.

A new model pins its [ontology preset](./modeling-lang/ontology.md) at creation: `--preset <file>`, else an
`ontology.json` next to the model file, else the built-in default ontology. The pin travels in the model file;
`--preset` on an existing model is rejected. `exec` is the JSON workflow — source projects mutate by text edit and
compile via the verbs below.

```
archi check [--project <dir> | --model <file>] [--json]
```

Compiles the project (or loads the model file) and reports
[findings](./modeling-lang/errors.md#errors-vs-findings). Compile diagnostics print as
`file:line:col: CODE: message` and exit 1; findings are advisory and exit 0. `--json` emits a structured envelope
(`{"status","findings"}` or `{"status","diagnostics"}`).

```
archi build [--project <dir>] [--emit-batch <file|->]
```

Compiles the project; with `--emit-batch`, writes the lowered statement batch as a JSON array — replayable through
`exec`, inspectable by agents.

```
archi nkp [--project <dir> | --model <file>] [--regime | --hotspots | --corridors] [--top | --scope <path>]
          [--exclude '<src> <rel> <dst>']... [--only <edge-type>]...
          [--tau-p <f>] [--tau-b <f>] [--neutrality degree|uniform] [--global-p <f>]
```

The [NKP landscape analysis](./scoring/nkp.md) over the model's epistatic slice. Output is JSON: the full report
by default, or one facet via `--regime` / `--hotspots` / `--corridors`.

## Output

Human-readable by default: results and cascades rendered in the
[surface syntax](./modeling-lang/source-format.md), findings and errors as one-liners carrying their `hint`.
`--json` emits the [agent interface](./agent-interface.md) response envelope instead — one contract, two skins.

## Exit codes

| code | meaning                                                             |
|------|----------------------------------------------------------------------|
| 0    | batch applied (or noop'd), successful read, or clean compile         |
| 1    | the batch was rejected or the project fails to compile; printed      |
| 2    | the invocation itself is malformed (unknown verb, bad flags)         |

## Locating the model

`check`, `build` and `nkp` locate their model by precedence: `--project <dir>`, then `--model <file>`, then the
nearest `archi.toml` upward from the working directory, then `archi.json`. A source project is compiled fresh on
every run — the source is the model. Remote/workspace discovery beyond this is distribution territory:
[saas](./distribution/saas.md), [on-prem](./distribution/on-prem.md).
