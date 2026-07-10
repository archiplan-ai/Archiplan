---
kind: functional
origin: intent
satisfied-by: [Compiler.Resolver, Compiler.Lower]
deferred:
---

# Ports declare the interface

A node's ports are declared at the node's definition, in one place — the definition
is the interface, and interface changes are diffs on the defining file. Every port
an edge or application uses must be declared on its node (`E_UNDECLARED_PORT`), and
`open` blocks cannot declare ports, so no other file can quietly widen an
interface.

```
def node AuthService:        // the node's interface: its declared ports, in one place
  port handle_login
  port handle_get_token
  port send_audit_log

def node LoginForm           // portless: no block needed
def node Orders.RefundHandler   // dotted form: augmentation into an existing scope
```

Lowering emits the declared ports on the node's `define` statement, so declared
ports exist in the model before any edge touches them. A port's connection type and
side are still fixed by its first use — declaration names the port, use types it —
and a declared, never-wired port is the `unused_port` finding, not an error
(`findings-never-block`): interface-first construction is exactly the state where
ports exist before their wiring.

The statement layer keeps creation-on-first-use for ports; declare-first is the
source discipline. A statement-built model that leans on use-created ports replays
via statements but does not round-trip through source
(`the-surface-lowers-to-one-batch`).

## System Context

The engine's port discipline — first use fixes type and side, conflicts reject —
predates the surface and stays where it is (`one-semantic-authority`); the source
format adds only the declaration gate in front of it. Reviewers read interfaces
from defining files, so an edge in a far-away module must never be able to invent a
port.

## Satisfy

`Compiler.Resolver` (checks every edge end and application port against the
declaring node's port set and raises `E_UNDECLARED_PORT` at the use's span) and
`Compiler.Lower` (puts the declared port set on each node's `define`, where port
claims compare as sets and divergent restatements reject).

- test — resolve::ports_must_be_declared
- test — parser::ports_are_rejected_in_open_blocks
- test — declared_ports::declared_ports_exist_before_any_edge
- test — declared_ports::first_use_fixes_type_and_side_of_a_declared_port
- test — declared_ports::port_claims_compare_as_sets
- test — declared_ports::dump_with_declared_ports_replays_idempotently
