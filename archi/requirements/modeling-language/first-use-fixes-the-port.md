---
kind: functional
origin: intent
satisfied-by: [Engine]
deferred:
---

# First use fixes the port

A port is the only place a connection lands, and it commits at first use: from the first
edge or application that names it, the port belongs to that connection type and — for a
directed type — that side, and a later use that disagrees rejects
(E_PORT_TYPE_CONFLICT, E_PORT_SIDE_CONFLICT). Several edges may share a port whenever
each matches its type and side. Declared ports exist from their node's definition on,
untyped until first use and surviving the loss of every edge — a declared, never-wired
port is the `unused_port` finding; a use-created port, the statement layer's door, lives
exactly as long as something attaches to it.

## System Context

Ports are the node's interface, and an interface whose meaning shifted per edge would
make connection types unreadable. First-use commitment keeps the discipline cheap because
each compile builds the model whole, so a binding never has to be released
(`ports-declare-the-interface` adds the source-side declaration gate in front of this
rule).

## Satisfy

`Engine` (fixes type and side at first use, rejects conflicts, shares matching ports, and
keeps declared ports alive edgeless as findings).

- test — errors::e_port_type_and_side_conflicts
- test — declared_ports::declared_ports_exist_before_any_edge
- test — declared_ports::first_use_fixes_type_and_side_of_a_declared_port
