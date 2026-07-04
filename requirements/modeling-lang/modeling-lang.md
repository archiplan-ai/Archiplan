# Modeling Language

A substrate to express arbitrary architectures in a structured way that can be transformed or queried later.

## Free graph

Spec consists of nodes and edges.

A node is not opaque: it may expose **ports** (named attachment points) and contain a **scope** (a nested subgraph of
inner nodes). Edges are not uniform either — every edge has a **kind** that decides where it may attach: outside a node,
on its ports, or across its scope boundary (see [Kinds](#kinds)).

### Identifiers

Every element has a stable identity and a human-readable handle, with a fixed contract between them:

- **id** — opaque, assigned at creation, immutable. Everything that must survive edits binds to ids: edge ends,
  patterns, requirement references, version diffs.
- **name** — the handle used in statements (`Payments`, `send_confirmation`). The name as written *is* the
  element's slug — the language imposes no casing convention. A name is unique among siblings in its scope; the
  same name may recur in different scopes and is addressed from outside by path (`Orders.ConfirmationHandler`).

`rename` rebinds a name and never touches the id, so renames are reference-safe by construction. Edges carry no
user-facing name: an edge is addressed *structurally*, by restating it — its identity is the tuple (kind, type,
ends/ports, carrier). Restating an existing edge refers to that edge rather than duplicating it.

## Nodes

### Ports

A **port** is a named attachment point on a node. Ports are the only place a
[connection](#connection) may land, and they form the node's interface to the rest of the graph.

Ports are named by the user at the point of use: a connection (or application) end attaches to a node through a
parenthesized port name — `Payments(send_confirmation)` — which creates the port on first use and reuses it
afterwards. A port is referenced as `node.<port>` (`Payments.send_confirmation`). Every port belongs to exactly one
connection type and, for directed types, one side (source or target), all fixed by its first use; a later use that
disagrees is rejected.

### Scope

A node contains a **scope**: a nested subgraph of inner nodes. Relations and connections operate between nodes at the
same level; an [application](#application) is what reaches across a node's scope boundary, mapping one of the node's
outer ports to a port of an inner node.

## Edges

### Views

A **view** is a named perspective on the model — one slice of edges that tells one story about the system: data
flows, dependencies, hierarchies, fault propagation. Edges are tagged with views at instantiation; a node belongs to
a view indirectly, through the edges incident to it. One node may appear in many views.

A view must be declared before use. An instantiation statement joins its edge to one or more views with `in`:

```
view data_flow;
view fault_prop;

rel fails_via := (Service type_of *) -> (Service type_of *);

Payments(send_message) send(PaymentConfirmation) Orders(handle_message) in data_flow;
Payments fails_via Orders in fault_prop;
Orders fails_via Shipping in fault_prop, data_flow;
```

- Tagging is per **edge**, not per type: edges of one type may live in different views, and one edge may belong to
  several views.
- Views of an existing edge are extended by restating it with `in`, and removed with `untag` (see [Language
  API](#language-api)).
- An untagged edge belongs to no view and is visible only to unfiltered queries.
- Referencing an undeclared view is rejected.
- Application edges are untagged plumbing: an application belongs to the views of the connection edges it routes.

Queries can slice the model by view (see [queries](./queries.md)).

### Kinds

Edges are *distinguished*: every edge belongs to exactly one of three kinds.

- **relation** — relates two nodes at their boundary.
- **connection** — attaches to a surface **port** on each end rather than to the node as a whole.
- **application** — maps an outer node's port to an inner node's port, crossing a node's scope boundary.

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

Derived pairs are **virtual** — only declared edges are stored. The transitive closure is computed at query time,
and only for queries that opt into it; analyses that count edges or node degrees see declared edges only.

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

**Arity follows the shape.** A binary type takes no carried node; a ternary type **requires** one at every
instantiation — written in parentheses after the conn slug (`send(PaymentConfirmation)`) and matched against the
carried slot's pattern. The carrier is optional in the grammar only so that one production covers both arities; in
use it is mandatory for ternary types and rejected for binary ones.

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
# type_of comes from the stdlib

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
Payments(send_confirmation) confirm(OrderId) Orders(handle_confirmation);

# inside Orders, the boundary port is delegated to an inner handler
node Orders {
  node ConfirmationHandler;
  handle_confirmation = ConfirmationHandler(handle_confirmation);
}
```

## Standard Library

Always available in any scope:

**type_of**

```
rel trans type_of := * -> *;
```

## Language API

User interacts with a model using statements separated with `;`. Interactive statements apply one at a time; a batch
submitted as one request is atomic. Every statement either applies, is an identical-restatement no-op, or fails with
a structured error that leaves the model untouched — see [errors](./errors.md).

**Write Operations:**
- node <Name> - define node
- view <Name> - declare a view
- rel <?trans> <Name> := <Pattern> - define relation type
- conn <Name> := <Pattern> - define connection type
- <Name> <rel> <Name> <?in ViewList> - instantiate a relation between nodes
- <Name>(<port>) <conn>(<?param>) <Name>(<port>) <?in ViewList> - create ports and connect them in place; <param> is
  the carried node — required when the conn type is ternary, forbidden when binary
- <port>(<?pattern>) = <Name>(<port>) - delegate an outer port to an inner node's port (application); the optional
  pattern routes only edges whose carried node matches it (see [Routing by carried node](#routing-by-carried-node))
- open <Path> - open a scope, where Path is a list of names separated with `.`
- <Name> { ... } - inline version of opening a scope, evaluating commands within it and then closing it

**Mutation Operations:**
- rename <Path> <NewName> - rebind a node's name; its id and every reference to it are untouched. Rejected with
  E_DUP_NAME if a sibling already uses the name
- delete <Path> - delete a node, its scope, and (recursively) everything that references any of it
- delete <edge-statement> - delete one edge, addressed by restating it: `delete Payments fails_via Orders`,
  `delete Payments(send_confirmation) confirm(OrderId) Orders(handle_confirmation)`,
  `delete handle_confirmation = ConfirmationHandler(handle_confirmation)`
- delete rel <Name> | delete conn <Name> - delete an edge type and all its edges
- delete view <Name> - delete a view; tagged edges just lose the tag
- untag <edge-statement> in <ViewList> - remove view tags from an edge (restating an edge with `in` adds tags)

Moving a node between scopes is deliberately absent: applications are scope-crossing by construction, so a move is a
delete + recreate under the new parent.

**Read Operations**

See [queries](./queries.md)

### Deletion semantics

`delete` always **cascades**: it removes the element together with the full closure of elements that reference it,
and the result reports everything removed, rendered as statements — nothing disappears silently.

Two integrity regimes govern what cascades and what merely drifts:

- **Reference integrity is hard.** The store never holds a reference to a deleted element. Deleting a node takes
  with it: edges ending on it or carrying it, applications delegating into it, type declarations whose patterns
  name it (and, transitively, those types' edges), and its whole inner scope.
- **Shape conformance is soft.** Shapes are checked when an edge is created. A later edit that erodes conformance —
  e.g. `delete Service type_of Payments` while `calls` edges on Payments rely on `(Service type_of *)` — succeeds;
  the nonconforming edges remain and are surfaced as [findings](./queries.md), not errors.

A port lives as long as some connection or application attaches to it; cascading away the last attached edge removes
the port and frees its name (a fresh first use may bind a new type or side). A delegated port left with no attached
connections is legal but suspect — a finding, not an error.

The [stdlib](#standard-library) cannot be deleted (E_STDLIB_PROTECTED).

## Cross-checks

Invariants an implementation must enforce; each is checkable at statement time.

- Several connection edges may attach to one port at the same time, provided each matches the port's connection type
  and side.
- A port's connection type and side are fixed by its first use and never change; a disagreeing use is rejected.
- A ternary connection edge is always instantiated with a carrier, and the carrier matches the carried slot's
  pattern; a binary connection edge never names a carrier.
- An application's outer port exists before the application does (some connection attaches to it).
- Two qualified delegations on one port must not match the same carried node.
- Every view referenced by `in` is declared.
- No stored element references a deleted element — `delete` removes the full referencing closure.
- A port exists iff at least one connection or application attaches to it.
- A failed statement leaves the model unchanged; an identical restatement is a no-op, never a duplicate.