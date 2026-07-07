# CLI

`archi` is a thin runner over the [modeling language](./modeling-lang/modeling-lang.md): it compiles
[`.arch` source projects](./modeling-lang/source-format.md) and renders results for humans. It adds no
vocabulary of its own. The source is the only source of truth — the CLI offers no JSON editing of the model:
mutation is a text edit to `.arch` source and a recompile, and the JSON
[statement layer](./modeling-lang/modeling-lang.md#statements) is the compiler's lowering target, not a
workflow.

## Commands

```
archi check [--project <dir>] [--json]
```

Compiles the project and reports
[findings](./modeling-lang/errors.md#errors-vs-findings). Compile diagnostics print as
`file:line:col: CODE: message` and exit 1; findings are advisory and exit 0. `--json` emits a structured envelope
(`{"status","findings"}` or `{"status","diagnostics"}`).

```
archi build [--project <dir>] [--emit-batch <file|->]
```

Compiles the project; with `--emit-batch`, writes the lowered statement batch as a JSON array — the
[deterministic lowering](./modeling-lang/source-format.md#lowering-and-determinism), inspectable by agents.

```
archi nkp [--project <dir>] [--regime | --hotspots | --corridors] [--top | --scope <path>]
          [--exclude '<src> <rel> <dst>']... [--only <edge-type>]...
          [--tau-p <f>] [--tau-b <f>] [--neutrality degree|uniform] [--global-p <f>]
```

The [NKP landscape analysis](./scoring/nkp.md) over the model's epistatic slice. Output is JSON: the full report
by default, or one facet via `--regime` / `--hotspots` / `--corridors`.

## Output

Human-readable by default: compile diagnostics as `file:line:col: CODE: message` lines, findings as one-liners
carrying their `hint`. `--json` emits structured output for tools instead; the full
[agent interface](./agent-interface.md) envelope belongs to that interface's transports, not the CLI.

## Exit codes

| code | meaning                                                             |
|------|----------------------------------------------------------------------|
| 0    | clean compile; findings, if any, are advisory                        |
| 1    | the project fails to compile; diagnostics printed                    |
| 2    | the invocation itself is malformed (unknown verb, bad flags)         |

## Locating the project

Every verb locates its project by precedence: `--project <dir>`, then the nearest `archi.toml` upward from the
working directory; finding neither is a usage error. The project is compiled fresh on every run — the source is
the model. Remote/workspace discovery beyond this is distribution territory:
[saas](./distribution/saas.md), [on-prem](./distribution/on-prem.md).
