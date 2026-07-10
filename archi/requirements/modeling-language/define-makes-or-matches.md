---
kind: functional
origin: intent
satisfied-by: [Engine]
deferred:
---

# Define makes or matches

`define` brings named elements — node, view, rel, conn — into existence, and never
silently alters one: an identical restatement is a `noop` reported as success, a
divergent definition rejects as E_REDECLARED with the existing definition in the error,
and no verb replaces or removes anything — replacement is a source edit and a recompile.
A creation statement carries the full path of what it creates; the prefix names the
container, which must already exist, so augmentation — writing into an existing node's
scope — is just a statement whose path lands inside it, and a missing container is
E_UNKNOWN_NAME, never an implicit create.

## System Context

Idempotent definitions plus structural edge identity (`an-edge-is-its-statement`) are
what make replays safe: recompiling unchanged source re-applies the whole batch as
no-ops, and at-least-once retries can never duplicate model state. The absence of
mutation vocabulary is owned by `source-is-the-only-truth`; this card owns what the one
creation verb means.

## Satisfy

`Engine` (executes `define` idempotently, reports noop on exact restatement, rejects
divergence with the stored actual, and resolves every path prefix before creating).

- test — semantics::define_is_idempotent
- test — errors::e_redeclared_on_divergent_define
- test — errors::e_unknown_name
- test — source_e2e::the_compiled_batch_replays_into_the_same_model
