---
kind: non-functional
origin: intent
satisfied-by: [Cli]
deferred:
---

# Findings never block

Errors reject; findings advise. Open requirements, pending stressors, unwired ports, decayed
evidence — the states a healthy workflow passes through — are surfaced on every check and never
fail it. The save that closes a stress round produces open requirements; blocking on them would
block the workflow on its own output.

## System Context

check and link verify run as CI gates; a gate that cries wolf gets deleted.

## Satisfy

`Cli` exits 0 on a clean compile whatever the findings say, and non-zero exactly when a source
fails to compile, a doc violates its schema, or the archive fails its seal.

- test — a model with a declared, unwired port checks clean at exit 0 with the finding printed
- test — an unresolvable satisfied-by path flips the same invocation to exit 1
