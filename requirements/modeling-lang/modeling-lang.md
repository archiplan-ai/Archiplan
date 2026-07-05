# Modeling Language

A substrate to express arbitrary architectures in a structured way that can be transformed or queried later.

## Free graph

Spec consists of nodes and edges.

A node is not opaque: it may expose **ports** (named attachment points) and contain a **scope** (a nested subgraph of
inner nodes). Edges are not uniform either — every edge has a **kind** that decides where it may attach: outside a node,
on its ports, or across its scope boundary (see [Kinds](#kinds)).

### Notation

The concrete syntax of the language is **JSON**: every statement is a JSON object, defined in
[Language API](#language-api). The compact notation used in examples throughout this spec —
`def node Payments;`, `Service type_of Payments;` — is illustrative **pseudo-syntax** only: each line is 1:1
shorthand for one statement object, used because JSON is hard to read in bulk. Nothing parses the pseudo-syntax;
interfaces may render statements this way for humans.

### Identifiers

Every element has a stable identity and a human-readable handle, with a fixed contract between them:

- **id** — opaque, assigned at creation, immutable. Everything that must survive edits binds to ids: edge ends,
  patterns, requirement references, version diffs.
- **name** — the handle used in statements (`Payments`, `send_confirmation`). The name as written *is* the
  element's slug — the language imposes no casing convention. A name is unique among siblings in its scope; the
  same name may recur in different scopes. Every node reference in a statement is an **absolute path** from the
  root (`Orders.ConfirmationHandler`) — there is no relative or context-dependent resolution.

`rename` rebinds a name and never touches the id, so renames are reference-safe by construction. Edges carry no
user-facing name: an edge is addressed *structurally*, by restating it — its identity is the tuple (kind, type,
ends/ports, carrier). Restating an existing edge refers to that edge rather than duplicating it.

## Nodes

### Ports

A **port** is a named attachment point on a node. Ports are the only place a
[connection](#connection) may land, and they form the node's interface to the rest of the graph.

Ports are named by the user at the point of use: a connection (or application) end names its port at the node —
`Payments(send_confirmation)` in pseudo-syntax, the `port` field of the end object in JSON — which creates the port
on first use and reuses it afterwards. A port is referenced as `node.<port>` (`Payments.send_confirmation`). Every
port belongs to exactly one connection type and, for directed types, one side (source or target), all fixed by its
first use; a later use that disagrees is rejected.

### Scope

A node contains a **scope**: a nested subgraph of inner nodes. Scope membership is expressed by path —
`Orders.ConfirmationHandler` lives in the scope of `Orders`. Relations and connections operate between nodes at the
same level; an [application](#application) is what reaches across a node's scope boundary, mapping one of the node's
outer ports to a port of an inner node.

## Edges

### Views

A **view** is a named perspective on the model — one slice of edges that tells one story about the system: data
flows, dependencies, hierarchies, fault propagation. Edges are tagged with views at instantiation; a node belongs to
a view indirectly, through the edges incident to it. One node may appear in many views.

A view must be declared before use. An edge statement joins its edge to one or more views with `in` (the `views`
field):

```
def view data_flow;
def view fault_prop;

def rel fails_via := (Service type_of *) -> (Service type_of *);

Payments(send_message) send(PaymentConfirmation) Orders(handle_message) in data_flow;
Payments fails_via Orders in fault_prop;
Orders fails_via Shipping in fault_prop, data_flow;
```

- Tagging is per **edge**, not per type: edges of one type may live in different views, and one edge may belong to
  several views.
- Views of an existing edge are extended by restating it with more views, and removed with the `untag` statement
  (see [Language API](#language-api)).
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

A typed kind of edge between two nodes. Declared with a `rel` statement, composing with
[transitivity](#transitivity), [direction](#direction) and [shape](#shape):

```
def rel trans type_of := * -> *;          # stdlib — pre-applied, shown for shape
def rel has_sensitive_data := (Service type_of *) -> (Data type_of *);
```

Use it:

```
def node Service;
def node Payments;

Service type_of Payments;    # instantiate relation `type_of` between nodes Service and Payments
```

##### Transitivity

A relation may be transitive (`trans`):
- `a -> b; b -> c => a -> c`
- `a <-> b; b <-> c => a <-> c`

Derived pairs are **virtual** — only declared edges are stored. The transitive closure is computed at query time,
and only for queries that opt into it; analyses that count edges or node degrees see declared edges only.

##### Direction

Direction is required on every type declaration:
- `->` — directed, source to target
- `<->` — non-directed; source/target order in edge statements carries no meaning

#### Connection

A typed kind of edge that connects nodes via concrete ports (in contrast to opaque [relation](#relation) edge).
Declared with a `conn` statement. A connection edge attaches to each node through a [port](#ports) named at the
point of use; the port is created on first use:

```
def conn send := (Service type_of *) (Message type_of *)-> (Service type_of *);
def conn confirm := (Service type_of *) -> (Service type_of *);

Payments(send_confirmation) confirm Orders(handle_confirmation);

def node PaymentConfirmation;
Message type_of PaymentConfirmation;

Payments(send_message) send(PaymentConfirmation) Orders(handle_message);
```

Several edges may attach to the same port — naming an existing port reuses it, provided the connection type and side
match (see [Ports](#ports)).

##### Shape

Shape restricts the set of nodes a connection of that type can connect. Each slot of a shape is a **pattern**:

- `*` — matches any node
- `(OrderId)` — matches exactly the node `OrderId`
- `(Service type_of *)` — matches any node `x` such that `Service type_of x`

Pattern anchors are absolute paths, bound by id at declaration time.

A. Binary shape `* <Direction> *`:
```
def conn calls := (Service type_of *) -> (Service type_of *);
```

B. Ternary shape `* (*)<Direction> *` — the middle node is *carried* by the connection edge:
```
def conn send := (Service type_of *) (Message type_of *)-> (Service type_of *);
```

**Arity follows the shape.** A binary type takes no carried node; a ternary type **requires** one at every
instantiation — `send(PaymentConfirmation)` — matched against the carried slot's pattern. A binary type rejects a
carrier at instantiation.

#### Application

An untyped kind of edge. Maps an outer node's port to an inner node's port, crossing the scope boundary. Inherits
direction from the ports it connects. There is one built-in application type — it is never declared, only used. The
statement names the delegating node explicitly (the outer port is written absolutely, `Orders.handle_confirmation`);
the inner node is a **direct child** of it, so the inner end is written as a bare child name:

```
def node Orders.ConfirmationHandler;

Orders.handle_confirmation = ConfirmationHandler(confirmations);   # outer boundary port realized by an inner port
```

The outer port must already exist (some connection attaches to it). The inner end follows the same rules as a
connection end: the named port is created or reused on the inner node, and inherits the outer port's connection type
and side.

##### Routing by carried node

The first way to split traffic is to attach it to differently named ports at the connection statements. When edges
carrying different concrete nodes genuinely share one port, a delegation can be qualified with a carried-node
[pattern](#shape) to route only the matching edges:

```
Payments(payment_events)  send(PaymentFailed) Orders(events);
Shipping(shipping_events) send(OrderCreated) Orders(events);

Orders.events(OrderCreated)  = OrderHandler(handle);    # only edges carrying OrderCreated
Orders.events(PaymentFailed) = PaymentHandler(handle);  # only edges carrying PaymentFailed
```

Resolution: an edge is routed by the qualified delegation whose pattern matches its carried node; edges matched by no
qualifier fall back to the unqualified delegation, if any. Two qualified delegations matching the same carried node
are rejected as ambiguous.

### Worked example (diagram (2))

One atomic batch:

```
def rel trans of_sort := * -> *;
# type_of comes from the stdlib

def node Functional;
def node Data;

def node Service;
Service of_sort Functional;
def node Payments;
def node Orders;

# Payments and Orders are concrete services (terms of type Service)
Service type_of Payments;
Service type_of Orders;

def node OrderId;
OrderId of_sort Data;

# a connection between two services' ports, carrying an OrderId
def conn confirm := (Service type_of *) (OrderId)-> (Service type_of *);
Payments(send_confirmation) confirm(OrderId) Orders(handle_confirmation);

# inside Orders, the boundary port is delegated to an inner handler
def node Orders.ConfirmationHandler;
Orders.handle_confirmation = ConfirmationHandler(handle_confirmation);
```

## Standard Library

Pre-applied in every model:

```
def rel trans type_of := * -> *;
```

Restating a stdlib definition identically is a no-op; deleting or divergently redefining it is rejected
(E_STDLIB_PROTECTED).

## Language API

A statement is a **JSON object**, discriminated by its `stmt` field — this section defines the actual language; the
pseudo-syntax elsewhere in this spec is shorthand for these objects. Statements cover the full operation set:
definitions, mutations and reads.

A model is edited by submitting a **batch**: an ordered JSON array of statements. A batch is atomic: every statement
either applies, is an identical-restatement no-op, or fails with a structured error that rolls the whole batch back
and reports the failing statement's index — see [errors](./errors.md). Reads may appear in batches; their output is
returned in the same per-statement results.

### Addressing — no ambient scope

Nothing in a statement depends on context:

- There is no session, no current scope, no lexical resolution. Every node reference is an **absolute path**.
- A creation statement carries the full path of the element it creates: the prefix names the container — which must
  already exist — and the last segment is the new name. `def node Orders.Auditor` says exactly where `Auditor`
  lives.
- An application names its delegating node absolutely; the one relative reference in the language is its inner end,
  a bare child name — the inner node is a direct child of the delegating node by rule, so its container is forced.
- **Augmentation** — adding something to an existing node's scope — is therefore just a statement whose path lands
  inside that node. It is legal iff the container exists; there is no way to accidentally create a container by
  writing into it, and no way to read a statement without knowing where it applies.

### Definitions

Named elements (`node`, `view`, `rel`, `conn`) are brought into existence by a **`define`** statement and replaced
by a **`redefine`** statement. The verb is the statement: `stmt` carries it, one **subject key** — `node`, `view`,
`rel` or `conn` — says what is defined and names it (a path for nodes, a name for the rest), and the definition's
parameters are sibling fields. Both verbs are **idempotent**: submitting the same statement twice leaves the same
model, the second application reporting `noop`.

| statement | precondition | effect                                                                                                        |
|-----------|--------------|-----------------------------------------------------------------------------------------------------------------|
| define    | —            | creates the element; an identical restatement is a no-op; an existing *divergent* definition is E_REDECLARED    |
| redefine  | must exist   | replaces the definition (below); a redefinition that changes nothing is a no-op; E_UNKNOWN_NAME if absent       |

`define` never silently alters the model: when the submitted definition contradicts the existing one — same name,
different body, or a rel/conn kind clash — it fails with E_REDECLARED and reports the existing definition.

`redefine` by element:

- **node** — keeps the node's id, name, ports and the edges attached to the node itself; **empties its scope**: the
  inner nodes and everything referencing them are removed as one cascade, reported in the result like a delete.
  Because batches are atomic, `redefine Orders` followed by fresh definitions inside `Orders` is a declarative
  "replace the internals" in one transaction — external wiring intact. An already-empty scope is a no-op.
- **rel / conn** — replaces the type's definition (transitivity, direction, shape). Existing edges are not
  re-checked eagerly: conformance is soft, and edges that no longer fit — including carrier arity mismatches —
  surface as [findings](./errors.md#errors-vs-findings), not errors.
- **view** — has no definition body; `redefine` does not apply (E_PARSE).

Edge statements (`rel-edge`, `conn-edge`, `app`) take no verb: an edge's identity is structural, stating it *is*
addressing it — a no-op, a view extension, or a fresh edge, never a duplicate. Together with idempotent definitions
this is what makes retries and whole-batch replays safe.

The distinction the two verbs pin down: `define` is **definition** (make this exist — finding it already there,
identical, is success), `redefine` is **re-definition** (replace what this name means), an addressed statement into
an existing scope is **augmentation** (add to what exists). None can be mistaken for another, and each fails loudly
when its precondition does not hold.

### Statements

**Definitions** — pseudo-syntax, then the statement objects:

```
def node Orders.RefundHandler;
def view data_flow;
def rel trans of_sort := * -> *;
def conn send := (Service type_of *) (Message type_of *)-> (Service type_of *);
redefine node Orders;
```

```json
{ "stmt": "define", "node": "Orders.RefundHandler" }
{ "stmt": "define", "view": "data_flow" }
{ "stmt": "define", "rel": "of_sort", "trans": true, "directed": true,
  "source": "*", "target": "*" }
{ "stmt": "define", "conn": "send", "directed": true,
  "source":  { "anchor": "Service", "rel": "type_of" },
  "carrier": { "anchor": "Message", "rel": "type_of" },
  "target":  { "anchor": "Service", "rel": "type_of" } }
{ "stmt": "redefine", "node": "Orders" }
```

`redefine` takes the same shape with `"stmt": "redefine"` (views excepted — no body to redefine). Arrows map to
`directed`: `->` is `true`, `<->` is `false`. Patterns map as `*` ↔ `"*"`, `(OrderId)` ↔ `{ "node": "OrderId" }`,
`(Service type_of *)` ↔ `{ "anchor": "Service", "rel": "type_of" }`. `trans` defaults to `false`; a `carrier` slot
makes the type ternary.

**Edges:**

```
Payments fails_via Orders in fault_prop;
Payments(send_message) send(PaymentConfirmation) Orders(handle_message) in data_flow;
Orders.events(OrderCreated) = OrderHandler(handle);
```

```json
{ "stmt": "rel-edge", "rel": "fails_via", "source": "Payments", "target": "Orders",
  "views": ["fault_prop"] }
{ "stmt": "conn-edge", "conn": "send",
  "source":  { "node": "Payments", "port": "send_message" },
  "carrier": "PaymentConfirmation",
  "target":  { "node": "Orders", "port": "handle_message" },
  "views": ["data_flow"] }
{ "stmt": "app", "node": "Orders", "port": "events",
  "route": { "node": "OrderCreated" },
  "inner": { "node": "OrderHandler", "port": "handle" } }
```

`views` is optional (absent = untagged); `carrier` is required for ternary types, rejected for binary ones; `route`
is the optional carried-node qualifier of a delegation.

**Mutations:**

```
rename Payments PaymentsGateway;
delete Orders;
delete Payments fails_via Orders;
delete rel fails_via;    delete conn send;    delete view fault_prop;
untag Payments fails_via Orders in fault_prop;
```

```json
{ "stmt": "rename", "node": "Payments", "to": "PaymentsGateway" }
{ "stmt": "delete", "node": "Orders" }
{ "stmt": "delete", "edge": { "stmt": "rel-edge", "rel": "fails_via",
                              "source": "Payments", "target": "Orders" } }
{ "stmt": "delete", "rel": "fails_via" }
{ "stmt": "untag", "edge": { "stmt": "rel-edge", "rel": "fails_via",
                             "source": "Payments", "target": "Orders" },
  "views": ["fault_prop"] }
```

- `rename` rebinds a node's name; its id and every reference to it are untouched. E_DUP_NAME if a sibling already
  uses the name.
- `delete` of a node removes it, its scope, and (recursively) everything that references any of it; of a type — the
  type and all its edges; of a view — the view only, tagged edges just lose the tag.
- An edge is addressed by **restating it**: the `edge` field is the edge's own statement object. A `views` field
  inside the restatement is ignored for addressing — views are not part of edge identity.
- Moving a node between scopes is deliberately absent: applications are scope-crossing by construction, so a move is
  a delete + recreate under the new parent.

**Reads** (see [queries](./queries.md)):

```
query types (Service) kinds (connection) scopes (Orders) in (data_flow);
check;
```

```json
{ "stmt": "query", "types": ["Service"], "kinds": ["connection"],
  "scopes": ["Orders"], "views": ["data_flow"] }
{ "stmt": "check" }
```

`query` slices the model with composed filters — each optional, absent meaning unrestricted: `types` keeps the
instances of the listed types (via the transitive `type_of` closure), `kinds` keeps edges of the listed kinds,
`views` keeps edges of the listed views plus the nodes related to them, `scopes` opens the named scopes (an empty
list is the top level only). The result is the slice as plain nodes and edges. `check` reports model-completeness
[findings](./errors.md#errors-vs-findings); `in` optionally restricts it to the edges of the named views.

### Deletion semantics

`delete` always **cascades**: it removes the element together with the full closure of elements that reference it,
and the result reports everything removed, rendered as statements — nothing disappears silently. A node `redefine`
resets the node's scope through the same machinery and reports the same way.

Two integrity regimes govern what cascades and what merely drifts:

- **Reference integrity is hard.** The store never holds a reference to a deleted element. Deleting a node takes
  with it: edges ending on it or carrying it, applications delegating into it, type declarations whose patterns
  name it (and, transitively, those types' edges), and its whole inner scope.
- **Shape conformance is soft.** Shapes are checked when an edge is created. A later edit that erodes conformance —
  e.g. deleting the classifier edge `Service type_of Payments` while `calls` edges on `Payments` rely on
  `(Service type_of *)` — succeeds; the nonconforming edges remain and are surfaced as
  [findings](./errors.md#errors-vs-findings), not errors.

A port lives as long as some connection or application attaches to it; cascading away the last attached edge removes
the port and frees its name (a fresh first use may bind a new type or side). A delegated port left with no attached
connections is legal but suspect — a finding, not an error.

The [stdlib](#standard-library) cannot be deleted (E_STDLIB_PROTECTED).

## Cross-checks

Invariants an implementation must enforce; each is checkable at statement time.

- Definition verbs are honored: `define` never alters an existing element (an identical restatement is a no-op, a
  divergent definition is rejected), `redefine` and `rename` never create one.
- Every path prefix in a statement resolves to an existing node — augmentation presupposes its container.
- Several connection edges may attach to one port at the same time, provided each matches the port's connection type
  and side.
- A port's connection type and side are fixed by its first use and never change; a disagreeing use is rejected.
- A ternary connection edge is always instantiated with a carrier, and the carrier matches the carried slot's
  pattern; a binary connection edge never names a carrier.
- An application's outer port exists before the application does (some connection attaches to it).
- An application's inner node is a direct child of its delegating node.
- Two qualified delegations on one port must not match the same carried node.
- Every view named in a `views` or `in` field is declared.
- No stored element references a deleted element — `delete` removes the full referencing closure.
- A port exists iff at least one connection or application attaches to it.
- A failed statement leaves the model unchanged; an identical restatement is a no-op, never a duplicate.
