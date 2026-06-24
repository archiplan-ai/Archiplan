# Modeling Language

A substrate to express arbitrary architectures in a structured way that can be transformed or queried later.

## Free graph

Spec consists of nodes and edges.

A node is not opaque: it may expose **ports** (named attachment points) and contain a **scope** (a nested subgraph of
inner nodes). Edges are not uniform either — every edge has a **kind** that decides where it may attach: outside a node,
on its ports, or across its scope boundary (see [Kinds](#kinds)).

### Identifiers

Each node and edge has unique identifier and a slug (human readable identifier;
id and slug can be one thing, depends on implementation).

## Nodes

### Ports

A **port** is a named attachment point on a node. Ports are the only place a
[connection](#connection) may land, and they form the node's interface to the rest of the graph.

Ports are named by the user at the point of use: a connection (or application) end attaches to a node through a
parenthesized port name — `Payments(send_confirmation)` — which creates the port on first use and reuses it
afterwards. 

### Scope

A node contains a **scope**: a nested subgraph of inner nodes. Relations and connections operate between nodes at the
same level; an [application](#application) is what reaches across a node's scope boundary, mapping one of the node's
outer ports to a port of an inner node.

## Edges

### Views

Edges are tagged with 

### Kinds

Edges are *distinguished*: every edge belongs to exactly one of three kinds.

- **relation** — relates two nodes at their boundary.
- **connection** - attaches to a surface **port** on each end rather than to the node as a whole.
- **application** - maps an outer node's port to an inner node's port, it crosses a node's scope boundary.

`relation` and `connection` are open kinds: users declare as many named types of them as they need. `application` is a
single built-in type (a singleton) — there are no user-declared application types, only application edges.

#### Relation

A typed kind of edge between two nodes. Declared with `rel`, composing with [transitivity](#transitivity), [direction](#direction) and
[shape](#shape):

```
rel trans type_of := * -> *;
rel has_sensitive_data := (Service type_of *) -> (Data type_of *)
```

Use it:

```
node Service
node Payments

Service type_of Payments # instantiate relation `type_of` between nodes Service and Payments 
```

##### Transitivity

A relation may be transitive:
- `a -> b; b -> c => a -> c`
- `a <-> b; b <-> c => a <-> c`

##### Direction

A relation may be either:
- `->` - directed
- `<->` - non-directed

#### Connection

A typed kind of edge that connects nodes via concrete ports (in contrast to opaque [relation](#relation) edge). 
Declared with `conn`. A connection statement attaches to each node through a [port](#ports), named
inline in parentheses; the port is created on first use:

```
conn send := (Service type_of *) (Message type_of *)-> (Service type_of *);
conn confirm := (Service type_of *) -> (Service type_of *);

Payments(send_confirmation) confirm Orders(handle_confirmation);

node PaymentConfirmation;
Message type_of PaymentConfirmation;

Payments(send_message) send(PaymentConfirmation) Orders(handle_message)
```

Several edges may attach to the same port — naming an existing port reuses it, provided the connection type and side
match (see [Ports](#ports)).

##### Direction

A connection may be either:
- `->` - directed
- `<->` - non-directed

##### Shape
Shape restricts the set of nodes a connection of that type can connect. Each slot of a shape is a parenthesized
**pattern**: `(Service type_of *)` matches any node `x` such that `Service type_of x`; a bare `(OrderId)` matches
exactly the node `OrderId`.

A. Binary shape `* <Direction> *`:
```
node Service;
conn calls := (Service type_of *) -> (Service type_of *)
```

B. Ternary shape `* (*)<Direction> *` — the middle node is *carried* by the connection edge. 
```
node Message;
conn send := (Service type_of *) (Message type_of *)-> (Service type_of *)
```

#### Application

An untyped kind of edge. Maps an outer node's port to an inner node's port, crossing the scope boundary.
Inherits direction from ports it connects.
There is one built-in `app` type — it is never declared, only used, inside the scope of the node whose port it delegates:

```
node Orders { # enter inner scope of node `Orders`
  node ConfirmationHandler;
  handle_confirmation = ConfirmationHandler(confirmations);   # outer boundary port realized by an inner port
}
```

The outer port must already exist (some connection attaches to it). The inner end follows the same rules as a
connection end: `(name)` creates or reuses a port on the inner node, which inherits the outer port's connection type
and side.

##### Routing by carried node

The first way to split traffic is to attach it to differently named ports at the connection statements. When edges
carrying different concrete nodes genuinely share one port, a delegation can be qualified with a carried-node
[pattern](#shape) to route only the matching edges:

```
Payments(payment_events)  send(PaymentFailed) Orders(events);
Shipping(shipping_events) send(OrderCreated) Orders(events);

node Orders {
  events(OrderCreated)  = OrderHandler(handle);    # only edges carrying OrderCreated
  events(PaymentFailed) = PaymentHandler(handle);  # only edges carrying PaymentFailed
}
```

Resolution: an edge is routed by the qualified delegation whose pattern matches its carried node; edges matched by no
qualifier fall back to the unqualified delegation, if any. Two qualified delegations matching the same carried node
are rejected as ambiguous.

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
  handle_confirmation = ConfirmationHandler(handle_confirmation);
}
```

#### Standard Library

Always available in any scope:

**type_of**

```
rel trans type_of := * -> *;
```

## Language API

User interacts with a model using statements separated with `;`.

**Write Operations:**
- node <Name> - define node
- rel <?trans> <Name> := <Pattern> - define relation type
- conn <Name> := <Pattern> - define relation type
- <Name> <rel> <Name> - instantiate a relation between nodes
- <Name>(<port>) <conn>(<?param>) <Name>(<port>) - create ports and connect them in place
- <port> = <Name>(<port>)
- open <Path> - open a scope, where Path is a list of names separated with `.`
- <Name> { ... } - inline version of opening a scope, evaluating commands within it and then closing it

**Read Operations**

See [queries](./queries.md)

## Cross-checks

- One port can be referenced by multiple connections at the same time