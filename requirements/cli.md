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

```
archi incidence [--project <dir>] [--session <slug> | --since <id>] [--exclude-pending]
                [--all-terms] [--json | --matrix | --k-hyper | --findings] [--no-matrix]
                [--kind <kind>]... [--min-severity info|warn|alert]
                [--tau-j <f>] [--tau-d <f>] [--depth <n>] [--path-limit <n>]
```

The [incidence analysis](./scoring/incidence.md): the stressor × component matrix of one stress session — or of
every session since a version — and its typed findings. The default output is the human report; `--json` and the
facet flags emit JSON. The same report auto-fires on the `archi version save` that closes a session. The
under-stressed sweep names behavioral terms by default; `--all-terms` widens it to the data vocabulary too.

```
archi read [<request.json> | -] [--at <id>] [--project <dir>]
```

The [agent read envelope](./agent-interface.md) over the CLI transport: a batch of read statements (`query`,
`check`) in — from a file, `-`, or piped stdin — and the response envelope out, verbatim. `--at <id>` runs the
batch against a version reconstructed from the sealed archive, which is how an agent grounds itself against a
plan's pin. Exit 0 on `ok`, 1 when a statement fails (`error.index` names it), 2 on a protocol error
(`E_BAD_REQUEST`).

```
archi query [--scope <path>]... [--type <path>]... [--kind <k>]... [--view <v>]...
            [--carrier <path>]... [--edge-type <name>]... [--top] [--at <id>] [--project <dir>]
```

One composed [subgraph query](./modeling-lang/queries.md): repeatable filter flags, the single `graph` result
unwrapped. An absent flag does not restrict; `--top` is the explicit empty `scopes` filter — the top level only.
`--carrier` slices the flow of a datum (the carrying edges and the nodes related to them, naming the node or a
classifying type); `--edge-type` slices by rel/conn type name. Errors print as one-liners, exit 1.

```
archi search <phrase>... [--kind element|intent|requirement|stressor|session]...
             [--limit <n>] [--json] [--project <dir>]
```

[Ranked retrieval](./search.md) by natural-language phrase across every KB object: model elements with their
identity prose, intents, requirements, stressors, sessions. Positional words join into one phrase; `--kind`
narrows (repeatable), `--limit` bounds the list (ten by default), `--json` emits the report envelope. Every hit
carries its address (slug or model path, `file:line` for docs) and its kind's next-hop refs. Unlike `query` and
`read`, search does not die with the model: a failed compile darkens only the element corpus — doc hits still
return, `dark` names the missing corpus, exit 0. Searching is always live-tree; there is no `--at` (the archive
seals the model alone).

## Output

Human-readable by default: compile diagnostics as `file:line:col: CODE: message` lines, findings as one-liners
carrying their `hint`. `--json` emits structured output for tools instead; `archi read` speaks the full
[agent interface](./agent-interface.md) envelope verbatim — the CLI is that contract's zero-infrastructure
transport.

## Exit codes

| code | meaning                                                             |
|------|----------------------------------------------------------------------|
| 0    | clean compile; findings, if any, are advisory; benign no-ops (an unchanged `version save`) are successes |
| 1    | the project fails to compile; diagnostics printed                    |
| 2    | the invocation itself is malformed (unknown verb, bad flags)         |

## Locating the project

Every verb locates its project by precedence: `--project <dir>`, then the nearest `archi.toml` upward from the
working directory; finding neither is a usage error. The project is compiled fresh on every run — the source is
the model. Remote/workspace discovery beyond this is distribution territory:
[saas](./distribution/saas.md), [on-prem](./distribution/on-prem.md).
