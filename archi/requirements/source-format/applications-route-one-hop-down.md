---
kind: functional
origin: intent
satisfied-by: [Compiler.Parser, Compiler.Resolver]
deferred:
---

# Applications route one hop down

An application delegates an outer port to a direct child's port — the right side is
always `Child.port`, one hop, per the core rule — and can route by carried-node
pattern, so one boundary port fans out to different handlers by payload. The left
side names the delegating node's port: bare inside the node's own block, `path.port`
otherwise.

```
handle_login = LoginHandler.handle             // inside a block: bare port of the block's node
AuthService.handle_login = LoginHandler.handle // flat form, e.g. at top level
events(OrderCreated) = OrderHandler.handle     // routed by carried-node pattern
```

The outer port must have a connection attached when the application applies
(`E_NO_OUTER_PORT` otherwise), and lowering sequences applications by their
delegation chains — the application that attaches a port lowers before the
applications delegating through it — so chains read outward-in wherever and in
whatever order they were authored (`renders-are-layout-blind` owns the ordering
claim). Reaching past a direct child, like joining nodes of different scopes, stays
the engine's `E_CROSS_SCOPE`; two qualified delegations matching the same carried
node stay its `E_AMBIGUOUS_DELEGATION`.

## System Context

Delegation semantics — one hop, attached-port precondition, routing by carrier —
are the engine's (`one-semantic-authority`); the surface adds the bare form inside
blocks and frees authors from ordering their chains. The auth example's
`handle_login = LoginHandler.handle` is the canonical use: a boundary port realized
by an inner port (`the-auth-example-compiles-as-written`).

## Satisfy

`Compiler.Parser` (accepts flat and bare forms and the `(route)` pattern; enforces
that the inner end is exactly `Child.port`) and `Compiler.Resolver` (resolves the
delegating node — enclosing block for bare, explicit path for flat — and checks the
child relationship and both ports against declared interfaces,
`ports-declare-the-interface`).

- test — parser::apps_parse_in_flat_and_block_form
- test — parser::app_inner_end_is_child_dot_port, parser::top_level_apps_name_the_delegating_node
- test — resolve::apps_check_children_and_ports
- test — source_e2e::delegation_chains_lower_outward_in_whatever_the_module_names
- test — semantics::qualified_delegations_route_by_carrier
