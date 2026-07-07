---
kind: non-functional
origin: intent
satisfied-by: [Compiler]
deferred:
---

# Located diagnostics

Every rejection — lexical, grammatical, or a semantic error raised deep in the engine — reads as
an ordinary compile error: `file:line:col`, a stable code, and a message that names the offender.
An agent must never have to guess which line of which file broke the build.

## System Context

Agents author most edits; their repair loop is only as good as the diagnostic that drives it.

## Satisfy

`Compiler` keeps a statement→span table through lowering, so engine rejections map back to the
source line that produced the statement; parse errors from several files are collected, not
first-error-only.

- test — source_e2e::engine_errors_localize_to_source_lines
- test — source_e2e::parse_errors_from_several_files_are_collected
