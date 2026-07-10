---
kind: functional
origin: intent
satisfied-by: [Compiler.Resolver]
deferred:
---

# Open reopens a scope

An `open` block re-opens an existing node's scope: definitions inside land in it,
and names inside resolve against it. Interface and internals can live in different
files — or the internals of one node can be split across several — and opens
resolve order-independently: an `open` waits until its target exists, wherever the
defining module sorts, so no file ordering can break a split.

```
open AuthService:            // AuthService must be visible here (own def or imported)
  def node Storage:
    port save_cred_hash
    port purge_cred
  def node LoginHandler:
    port handle
    port persist
  LoginHandler.persist store(->CredHash) Storage.save_cred_hash
  handle_login = LoginHandler.handle
```

The target must be visible where the `open` stands — the module's own definition or
an imported one (`imports-are-visibility-gates`) — and an `open` whose target never
comes to exist is reported. Opens declare nothing themselves: no ports
(`ports-declare-the-interface`), no definition comment (`open` lines take none),
and reopening a scope any number of times is free — `open` is scope access, not
redeclaration, which is why it escapes `E_REDECLARED`.

## System Context

One definition site per name is the project-wide law, yet a node's internals
routinely outgrow the file that declares its interface. `open` is the pressure
valve: the auth worked example keeps `AuthService`'s interface in `auth.arch` and
its internals in `auth_internals.arch` (`the-auth-example-compiles-as-written`),
and lowering guarantees the split never shows in the batch
(`renders-are-layout-blind`).

## Satisfy

`Compiler.Resolver` (resolves each `open` against the semantic tree built from all
modules, iterating until targets exist regardless of module order; definitions
inside an `open` land on the target's path; unresolvable opens are reported at
their span).

- test — resolve::opens_resolve_across_files_in_any_order
- test — resolve::unresolvable_opens_are_reported
- test — resolve::cross_file_defs_opens_and_flows_resolve
- test — parser::open_blocks_nest_defs_edges_and_apps
