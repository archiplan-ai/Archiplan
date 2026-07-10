---
kind: functional
origin: intent
satisfied-by: [Cli, Compiler]
deferred:
---

# Every verb finds its project

Every verb but `init` locates its project by one precedence: `--project <dir>`, else the
nearest `archi.toml` upward from the working directory; finding neither is a usage error.
`init` is the verb before that precedence exists — it takes its target as a positional
argument, and handing it `--project` is malformed. The project is compiled fresh on every
run: the source is the model, and no verb trusts a leftover artifact.

## System Context

Verbs run from editors, hooks, CI and nested directories; a project guessed differently
per caller would make every report ambiguous. `the-manifest-marks-the-root` defines what
the root is; `source-is-the-only-truth` is why nothing caches between runs.

## Satisfy

`Cli` (flag-then-upward resolution and the init exception). `Compiler` (a fresh compile
per invocation through its `load_sources` door).

- test — init_e2e::a_second_init_changes_no_bytes_and_extra_args_are_usage_errors
- test — init_e2e::the_verbs_around_init_keep_their_contracts
- test — source_e2e::compilation_is_deterministic_under_source_order
