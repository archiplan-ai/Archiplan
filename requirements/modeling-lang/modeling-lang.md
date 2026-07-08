# Modeling Language

A substrate to express arbitrary architectures in a structured way that can be transformed or queried later.

## Free graph

Spec consists of nodes and edges.

A node is not opaque: it may expose **ports** (named attachment points) and contain a **scope** (a nested subgraph of
inner nodes). Edges are not uniform either — every edge has a **kind** that decides where it may attach: outside a node,
on its ports, or across its scope boundary (see [Kinds](#kinds)).

### Notation

The language has two concrete syntaxes over one semantic core:

- **JSON statements** — the machine API, defined in [Language API](#language-api): every statement is a JSON
  object; batches are atomic. This is what source compiles down to and what agents read through — the language's
  machine layer, not an editing surface.
- **`.arch` source** — the human syntax, defined in [source-format.md](./source-format.md): a project of diffable
  text files with modules, imports and lexical scopes, compiled down to the same statements. The source is the
  **only source of truth**: the language is declarative-only (no mutation vocabulary — edits are text edits and a
  recompile); the compact notation used in examples throughout this spec (`def node Payments`,
  `Service type_of Payments`) is exactly this surface syntax, and dumps render in it.

### Identifiers

Every element has a stable identity and a human-readable handle, with a fixed contract between them:

- **id** — opaque, assigned at creation, immutable. Everything that must survive edits binds to ids: edge ends,
  patterns, requirement references, version diffs.
- **name** — the handle used in statements (`Payments`, `send_confirmation`). The name as written *is* the
  element's slug — the language imposes no casing convention. A name is unique among siblings in its scope; the
  same name may recur in different scopes. Every node reference in a statement is an **absolute path** from the
  root (`Orders.ConfirmationHandler`) — there is no relative or context-dependent resolution.

Edges carry no user-facing name: an edge is addressed *structurally*, by restating it — its identity is the tuple
(kind, type, ends/ports, carrier). Restating an existing edge refers to that edge rather than duplicating it.

## Nodes

### Ports

A **port** is a named attachment point on a node. Ports are the only place a
[connection](#connection) may land, and they form the node's interface to the rest of the graph.

Ports come into existence two ways:

- **Declared** — listed on the node's definition (`ports` field; `port` lines of a `def node` block in source).
  A declared port is part of the interface: it exists before any edge and survives losing its last edge. The
  [source format](./source-format.md) requires every used port to be declared; a declared port nothing attaches to
  is the `unused_port` [finding](./errors.md#errors-vs-findings). A `define` restating a node compares `ports` as
  a set when the field is present (divergence is E_REDECLARED) and makes no claim when absent.
- **Use-created** — named at the point of use: a connection (or application) end names its port at the node — the
  `port` field of the end object — which creates the port on first use and reuses it afterwards. A use-created
  port lives only as long as something attaches to it.

A port is referenced as `node.<port>` (`Payments.send_confirmation`). Every port belongs to at most one connection
type and, for directed types, one side (source or target), fixed by its **first use** — a declared port is untyped
until then; a later use that disagrees is rejected.

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
def view data_flow
def view fault_prop

def rel fails_via := (Service type_of *) -> (Service type_of *)

Payments.send_message send(PaymentConfirmation) Orders.handle_message in data_flow
Payments fails_via Orders in fault_prop
Orders fails_via Shipping in fault_prop, data_flow
```

- Tagging is per **edge**, not per type: edges of one type may live in different views, and one edge may belong to
  several views.
- Views of an existing edge are extended by restating it with more views; removing a tag is a source edit and a
  recompile.
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
def rel trans type_of := * -> *          // stdlib — pre-applied, shown for shape
def rel has_sensitive_data := (Service type_of *) -> (Data type_of *)
```

Use it:

```
def node Service
def node Payments

Service type_of Payments    // instantiate relation `type_of` between nodes Service and Payments
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
Declared with a `conn` statement. A connection edge attaches to each node through a [port](#ports) — declared on
the node or created at first use:

```
def conn send := (Service type_of *) ->(Message type_of *) (Service type_of *)
def conn confirm := (Service type_of *) -> (Service type_of *)

Payments.send_confirmation confirm Orders.handle_confirmation

def node PaymentConfirmation
Message type_of PaymentConfirmation

Payments.send_message send(PaymentConfirmation) Orders.handle_message
```

Several edges may attach to the same port — naming an existing port reuses it, provided the connection type and side
match (see [Ports](#ports)).

##### Shape

Shape restricts the set of nodes a connection of that type can connect. Each slot of a shape is a **pattern**:

- `*` — matches any node
- `OrderId` — matches exactly the node `OrderId`
- `(Service type_of *)` — matches any node `x` such that `Service type_of x`

Pattern anchors are absolute paths, bound by id at declaration time.

A shape is `source LANES target`. Direction means **initiation**; carried slots ride on the lanes, one per
direction:

| shape | meaning | fields |
|-------|---------|--------|
| `* -> *` | directed, no payload | — |
| `* ->P *` | directed, forward payload | `carrier` |
| `* ->P, <-Q *` | request/response: `P` out, `Q` back | `carrier`, `rev_carrier` |
| `* ->, <-Q *` | pull: initiation forward, payload back | `rev_carrier` |
| `* <-> *` | undirected | — |
| `* <->P *` | undirected, single payload | `carrier` |

```
def conn calls := (Service type_of *) -> (Service type_of *)
def conn send  := (Service type_of *) ->(Message type_of *) (Service type_of *)
def conn login := * ->LoginForm, <-AuthResponse *
```

A reverse carried slot requires a directed type (`<->` has no lanes to tell apart); port sides are unchanged — the
source port stays `source` even when payload flows back.

**Arity follows the lanes.** A lane with a carried slot **requires** a concrete carried node at every
instantiation — `send(PaymentConfirmation)`, `login(->LoginForm, <-AuthResponse)` — matched against that slot's
pattern; a lane without one rejects it. In source, a lane whose pattern is an exact node may be omitted at the edge
and defaults to it ([source-format.md](./source-format.md#connection-edges)); the statement layer always names
carriers explicitly.

#### Application

An untyped kind of edge. Maps an outer node's port to an inner node's port, crossing the scope boundary. Inherits
direction from the ports it connects. There is one built-in application type — it is never declared, only used. The
statement names the delegating node explicitly (the outer port is written absolutely, `Orders.handle_confirmation`);
the inner node is a **direct child** of it, so the inner end is written as a bare child name:

```
def node Orders.ConfirmationHandler

Orders.handle_confirmation = ConfirmationHandler.confirmations   // outer boundary port realized by an inner port
```

The outer port must already exist (some connection attaches to it). The inner end follows the same rules as a
connection end: the named port is created or reused on the inner node, and inherits the outer port's connection type
and side.

##### Routing by carried node

The first way to split traffic is to attach it to differently named ports at the connection statements. When edges
carrying different concrete nodes genuinely share one port, a delegation can be qualified with a carried-node
[pattern](#shape) to route only the matching edges:

```
Payments.payment_events  send(PaymentFailed) Orders.events
Shipping.shipping_events send(OrderCreated) Orders.events

Orders.events(OrderCreated)  = OrderHandler.handle    // only edges carrying OrderCreated
Orders.events(PaymentFailed) = PaymentHandler.handle  // only edges carrying PaymentFailed
```

Resolution: an edge is routed by the qualified delegation whose pattern matches its carried node; edges matched by no
qualifier fall back to the unqualified delegation, if any. Two qualified delegations matching the same carried node
are rejected as ambiguous.

### Worked example (diagram (2))

One atomic batch:

```
def rel trans of_sort := * -> *
// type_of comes from the stdlib

def node Functional
def node Data

def node Service
Service of_sort Functional
def node Payments
def node Orders

// Payments and Orders are concrete services (terms of type Service)
Service type_of Payments
Service type_of Orders

def node OrderId
OrderId of_sort Data

// a connection between two services' ports, carrying an OrderId
def conn confirm := (Service type_of *) ->OrderId (Service type_of *)
Payments.send_confirmation confirm(OrderId) Orders.handle_confirmation

// inside Orders, the boundary port is delegated to an inner handler
def node Orders.ConfirmationHandler
Orders.handle_confirmation = ConfirmationHandler.handle_confirmation
```

## Standard Library

Pre-applied in every model:

```
def rel trans type_of := * -> *
```

Restating a stdlib definition identically is a no-op; a divergent definition is rejected (E_REDECLARED), and
tagging a stdlib edge into views is rejected (E_STDLIB_PROTECTED) — tags on it would not survive a dump replay.

## Language API

A statement is a **JSON object**, discriminated by its `stmt` field — this section defines the actual language; the
pseudo-syntax elsewhere in this spec is shorthand for these objects. Statements cover definitions, edges and reads.
There is **no mutation vocabulary** — no rename, delete, untag or redefine: a model changes only by editing its
`.arch` source and recompiling.

A model is built by executing a **batch**: an ordered JSON array of statements — the
[compiler's lowered output](./source-format.md#lowering-and-determinism), or reads submitted through the
[agent interface](../agent-interface.md). A batch is atomic: every statement either applies, is an
identical-restatement no-op, or fails with a structured error that rolls the whole batch back and reports the
failing statement's index — see [errors](./errors.md). Reads may appear in batches; their output is returned in
the same per-statement results.

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

Named elements (`node`, `view`, `rel`, `conn`) are brought into existence by a **`define`** statement. The verb is
the statement: `stmt` carries it, one **subject key** — `node`, `view`, `rel` or `conn` — says what is defined and
names it (a path for nodes, a name for the rest), and the definition's parameters are sibling fields. `define` is
**idempotent**: submitting the same statement twice leaves the same model, the second application reporting `noop`.

| statement | precondition | effect                                                                                                        |
|-----------|--------------|-----------------------------------------------------------------------------------------------------------------|
| define    | —            | creates the element; an identical restatement is a no-op; an existing *divergent* definition is E_REDECLARED    |

`define` never silently alters the model: when the submitted definition contradicts the existing one — same name,
different body, or a rel/conn kind clash — it fails with E_REDECLARED and reports the existing definition. There is
no re-definition verb: replacing what a name means is a **source edit** — change the `def` line and recompile.

Every subject also takes an optional **`doc`** field — the element's prose definition: one identity sentence, at
most 240 characters, free of obligation vocabulary
([source-format.md#definitions](./source-format.md#definitions)); node defines additionally take **`port_docs`**,
a map from declared ports to their definitions (it requires `ports` and may name only its entries). Text is
normalized (whitespace collapsed) and validated at the schema with the same shared rule the source comment gate
applies, so a stored definition always re-parses from a render. `doc` participates in identity like every other
parameter: an omitted `doc` makes no claim, a divergent one is E_REDECLARED.

Edge statements (`rel-edge`, `conn-edge`, `app`) take no verb: an edge's identity is structural, stating it *is*
addressing it — a no-op, a view extension, or a fresh edge, never a duplicate. Together with idempotent definitions
this is what makes whole-batch replays safe — a recompile of unchanged source re-applies the same batch as all
no-ops.

The distinction that remains: `define` is **definition** (make this exist — finding it already there, identical, is
success), and an addressed statement into an existing scope is **augmentation** (add to what exists). Neither can be
mistaken for the other, and each fails loudly when its precondition does not hold.

### Statements

**Definitions** — surface syntax, then the statement objects:

```
def node Orders.RefundHandler
def node AuthService:
  port handle_login
def view data_flow
def rel trans of_sort := * -> *
def conn send := (Service type_of *) ->(Message type_of *) (Service type_of *)
def conn login := * ->LoginForm, <-AuthResponse *
```

```json
{ "stmt": "define", "node": "Orders.RefundHandler" }
{ "stmt": "define", "node": "AuthService", "ports": ["handle_login"] }
{ "stmt": "define", "view": "data_flow" }
{ "stmt": "define", "rel": "of_sort", "trans": true, "directed": true,
  "source": "*", "target": "*" }
{ "stmt": "define", "conn": "send", "directed": true,
  "source":  { "anchor": "Service", "rel": "type_of" },
  "carrier": { "anchor": "Message", "rel": "type_of" },
  "target":  { "anchor": "Service", "rel": "type_of" } }
{ "stmt": "define", "conn": "login", "directed": true,
  "source": "*",
  "carrier": { "node": "LoginForm" },
  "rev_carrier": { "node": "AuthResponse" },
  "target": "*" }
```

Arrows map to `directed`: `->` is `true`, `<->` is `false`. Patterns map as `*` ↔ `"*"`, `OrderId` ↔ `{ "node": "OrderId" }`,
`(Service type_of *)` ↔ `{ "anchor": "Service", "rel": "type_of" }`. `trans` defaults to `false`. `carrier` is the
forward lane's carried slot, `rev_carrier` the reverse lane's ([Shape](#shape)); `rev_carrier` requires
`directed: true`. `ports` lists the node's [declared ports](#ports): absent means no claim, present compares as a
set.

**Edges:**

```
Payments fails_via Orders in fault_prop
Payments.send_message send(PaymentConfirmation) Orders.handle_message in data_flow
UI.login login(->LoginForm, <-AuthResponse) AuthService.handle_login
Orders.events(OrderCreated) = OrderHandler.handle
```

```json
{ "stmt": "rel-edge", "rel": "fails_via", "source": "Payments", "target": "Orders",
  "views": ["fault_prop"] }
{ "stmt": "conn-edge", "conn": "send",
  "source":  { "node": "Payments", "port": "send_message" },
  "carrier": "PaymentConfirmation",
  "target":  { "node": "Orders", "port": "handle_message" },
  "views": ["data_flow"] }
{ "stmt": "conn-edge", "conn": "login",
  "source":  { "node": "UI", "port": "login" },
  "carrier": "LoginForm",
  "rev_carrier": "AuthResponse",
  "target":  { "node": "AuthService", "port": "handle_login" } }
{ "stmt": "app", "node": "Orders", "port": "events",
  "route": { "node": "OrderCreated" },
  "inner": { "node": "OrderHandler", "port": "handle" } }
```

`views` is optional (absent = untagged); `carrier`/`rev_carrier` are each required exactly when the type's
corresponding lane has a carried slot ([Shape](#shape)); `route` is the optional carried-node qualifier of a
delegation, matched against the **forward** carrier.

**No mutations.** There is no rename, delete, untag or redefine statement — the language cannot alter or remove
what exists, only create and read it. Every change to a model is a [source edit](./source-format.md) and a
recompile: renaming is editing the `def` line, removal is deleting the lines, retagging is editing the `in`
clause, and the git diff is the change record. Reference integrity therefore never has to be repaired
incrementally — each compile builds the model whole from source, so a reference to something the source no longer
defines is a compile error ([E_UNKNOWN_NAME](./errors.md#catalog)), not a dangling pointer.

**Reads** (see [queries](./queries.md)):

```
query types (Service) kinds (connection) scopes (Orders) in (data_flow)
check
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

### Integrity

A model is built whole on every compile — there is nothing to delete in place, nothing to cascade, no persisted
state to drift. Two integrity regimes govern how a compiled model relates to its source:

- **Reference integrity is hard.** The store never holds a reference to an element the source does not define. A
  statement naming a missing node, type or view is [E_UNKNOWN_NAME](./errors.md#catalog) at compile — so removing a
  definition from source without removing what references it fails loudly. Removal is therefore total and manual:
  drop the `def` line and every line that names it, and the diff records exactly what left.
- **Shape conformance is checked at creation.** An edge is validated against its type's patterns when it is
  created. The [deterministic lowering](./source-format.md#lowering-and-determinism) lands classifier edges before
  the shapes that consult them, so a fresh compile checks every edge against fully-populated patterns: a shape that
  no longer fits is a compile error at the offending source line, not a lingering nonconformance. (In-place edits
  could once erode conformance and surface as drift findings; whole-compile rebuilds leave no room for that state.)

Port lifecycle follows the source, not a mutation history. A **use-created** port exists iff at least one
connection or application in the source attaches to it — named at a point of use. A **declared** port exists from
its node's definition on; a declared port nothing attaches to is the `unused_port`
[finding](./errors.md#errors-vs-findings), not an error.

The [stdlib](#standard-library) cannot be tagged into views (E_STDLIB_PROTECTED); a divergent redefinition of a
stdlib name is an ordinary E_REDECLARED.

## Cross-checks

Invariants an implementation must enforce; each is checkable at statement time.

- `define` never alters an existing element: an identical restatement is a no-op, a divergent definition is
  rejected. There is no verb that replaces or removes one — that is a source edit.
- Every path prefix in a statement resolves to an existing node — augmentation presupposes its container.
- Several connection edges may attach to one port at the same time, provided each matches the port's connection type
  and side.
- A port's connection type and side are fixed by its first use in the lowered batch; a disagreeing later use is
  rejected. (Each compile is fresh, so the binding never has to be released.)
- Each lane of a connection edge is instantiated with a carried node exactly when the type's lane has a carried
  slot, and the node matches that slot's pattern; a reverse slot requires a directed type.
- An application's outer port exists and has an attachment before the application does (a declared port with
  nothing attached cannot delegate yet).
- An application's inner node is a direct child of its delegating node.
- Two qualified delegations on one port must not match the same carried node.
- Every view named in a `views` or `in` field is declared.
- No stored element references a missing one — a reference the source does not define is a compile error.
- A use-created port exists iff at least one connection or application attaches to it; a declared port exists from
  its node's definition on.
- A failed statement leaves the model unchanged; an identical restatement is a no-op, never a duplicate.
