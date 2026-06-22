# Modeling Language

A substrate to express arbitrary architectures in a structured way that can be transformed or queried later.

## Textual syntax

Constructs can be expressed syntactically.

## Free graph

Spec consists of nodes and edges, each node and edge has unique identifier and a slug (human readable identifier;
id and slug can be one thing, depends on implementation).

A node is not opaque: it may expose **ports** (named attachment points) and contain a **scope** (a nested subgraph of
inner nodes). Edges are not uniform either — every edge has a **kind** that decides where it may attach: outside a node,
on its ports, or across its scope boundary (see [Kinds](#kinds)).

## Nodes

### Ports

A **port** is a named attachment point on a node. Ports are the only place a
[connection](#connection) may land, and they form the node's interface to the rest of the graph.

Ports are named by the user at the point of use: a connection (or application) end attaches to a node through a
parenthesized port name — `Payments(send_confirmation)` — which creates the port on first use and reuses it
afterwards. A port is referenced as `node.<port>` (`Payments.send_confirmation`). Every port belongs to exactly one
connection type and, for directed types, one side (source or target), all fixed by its first use; a later use that
disagrees is rejected.

The name is optional: a bare end (`Payments confirm Orders`) attaches to the node's **default port** for that
connection type and side, named by the conn slug itself. Naming is only needed to distinguish — when a node both
sends and receives the same connection type, or handles different traffic on different ports.

### Scope

A node contains a **scope**: a nested subgraph of inner nodes. Relations and connections operate between nodes at the
same level; an [application](#application) is what reaches across a node's scope boundary, mapping one of the node's
outer ports to a port of an inner node.

## Edges

### Types

Edges have explicit `type` property, a type is defined by pair (TypeId, Shape).

#### Direction

- `->` - directed
- `<->` - non-directed

#### Transitivity

An edge is either transitive or not:
- `a -> b; b -> c => a -> c`
- `a <-> b; b <-> c => a <-> c`

#### Shape
Shape restricts the set of nodes an edge of that type can connect. Each slot of a shape is a parenthesized
**pattern**: `(Service type_of *)` matches any node `x` such that `Service type_of x`; a bare `(OrderId)` matches
exactly the node `OrderId`.

A. Binary shape `* -> *`:
```
node Data;
node Service;
rel trans type_of := * -> *;
rel has_sensitive_data := (Service type_of *) -> (Data type_of *)
```

B. Ternary shape `* (*)-> *` — the middle node is *carried* by the edge. Connections use it to name the ports they
generate (see [Connections](#connection)).
```
node Message;
conn send := (Service type_of *) (Message type_of *)-> (Service type_of *)
```

### Kinds

Edges are *distinguished*: every edge belongs to exactly one of three kinds.

- **relation** is external — it relates two nodes at their boundary and never enters a scope.
- **connection** penetrates nodes' scopes — it attaches to a **port** on each end rather than to the node as a whole.
- **application** maps an outer node's port to an inner node's port — it crosses a node's scope boundary.

`relation` and `connection` are open kinds: users declare as many named types of them as they need. `application` is a
single built-in type (a singleton) — there are no user-declared application types, only application edges.

#### Relation

External, node ↔ node. Declared with `rel`, composing with [transitivity](#transitivity), [direction](#direction) and
[shape](#shape):

```
rel trans type_of := * -> *;
rel has_sensitive_data := (Service type_of *) -> (Data type_of *)
```

#### Connection

Port ↔ port. Declared with `conn`. A connection statement attaches to each node through a [port](#ports), named
inline in parentheses; the port is created on first use:

```
conn send := (Service type_of *) (Message type_of *)-> (Service type_of *);
conn confirm := (Service type_of *) -> (Service type_of *);

Payments(send_confirmation) confirm Orders(handle_confirmation);
#   ⇒ ports  Payments.send_confirmation  and  Orders.handle_confirmation

Payments confirm Orders;
#   ⇒ default ports  Payments.confirm  and  Orders.confirm
```

Several edges may attach to the same port — naming an existing port reuses it, provided the connection type and side
match (see [Ports](#ports)).

#### Application

Maps an outer node's port to an inner node's port, crossing the scope boundary. There is one built-in `app` type — it
is never declared, only used, inside the scope of the node whose port it delegates:

```
node Orders {
  node ConfirmationHandler;
  app Orders.handle_confirmation = ConfirmationHandler(confirmations);   # outer boundary port realized by an inner port
}
```

The outer port must already exist (some connection attaches to it). The inner end follows the same rules as a
connection end: `(name)` creates or reuses a port on the inner node, which inherits the outer port's connection type
and side.

##### Delegation shorthand

When the inner port should simply mirror the outer one — the common case — the application collapses to a delegation
by port name:

```
app handle_confirmation -> ConfirmationHandler;
#   ≡ app Orders.handle_confirmation = ConfirmationHandler(handle_confirmation)
```

A default port of a directed connection type is delegated by its slug, with the arrow picking the side; an undirected
type has a single default port per node, so `->` simply delegates it:

```
app confirm -> ConfirmationHandler;   # target side: Orders.confirm realized by the inner node
app confirm <- ConfirmationHandler;   # source side: the inner node emits through Orders.confirm
```

##### Routing by carried node

The first way to split traffic is to attach it to differently named ports at the connection statements. When edges
carrying different concrete nodes genuinely share one port, a delegation can be qualified with a carried-node
[pattern](#shape) to route only the matching edges:

```
Payments(payment_events)  send Orders(events);
Shipping(shipping_events) send Orders(events);

node Orders {
  app events(OrderCreated)  -> OrderHandler;    # only edges carrying OrderCreated
  app events(PaymentFailed) -> PaymentHandler;  # only edges carrying PaymentFailed
  app events -> Audit;                          # fallback: every other edge on the port
}
```

Resolution: an edge is routed by the qualified delegation whose pattern matches its carried node; edges matched by no
qualifier fall back to the unqualified delegation, if any. Two qualified delegations matching the same carried node
are rejected as ambiguous.

The explicit form takes the same qualifier on the outer port:
`app Orders.events(OrderCreated) = OrderHandler(order_created);`

### Worked example (diagram (2))

Textual interface works as repl:

```
rel trans of_sort := * -> *;
rel trans type_of := * -> *;

node Functional;
node Data;

node Service;
Service of_sort Functional;
node Payments;
node Orders;

# Payments and Orders are concrete services (terms of type Service)
Service type_of Payments;
Service type_of Orders;

node OrderId;
OrderId of_sort Data;

# a connection between two services' ports, carrying an OrderId
conn confirm := (Service type_of *) (OrderId)-> (Service type_of *);
Payments(send_confirmation) confirm Orders(handle_confirmation);

# inside Orders, the boundary port is delegated to an inner handler
node Orders {
  node ConfirmationHandler;
  app handle_confirmation -> ConfirmationHandler;
  # ≡ app Orders.handle_confirmation = ConfirmationHandler(handle_confirmation)
}
```

### Introspection

The REPL can enumerate a node's ports, so existing names never have to be guessed:

```
> ports Orders
Orders.handle_confirmation   (conn confirm, target side, carries OrderId)
                             <= Payments.send_confirmation
                             => ConfirmationHandler.handle_confirmation   (app)
```

#### Standard Library

Archiplan provides standard library of edge types to enable [scoring](../scoring/)
