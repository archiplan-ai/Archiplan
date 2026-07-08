# Source Format

Model definitions stored as **source code**: a project of `.arch` files that is diffable, modular and reviewable.
The source *is* the model — tools compile the project fresh on every run; there is no build artifact to keep in
sync. Mutation vocabulary does not exist — not in source and not in the
[statement layer](./modeling-lang.md#language-api): a mutation is a text edit and a recompile, and the diff is the
change record.

The surface language is **sugar over the statement layer**: every construct lowers to the absolute-path JSON
statements of [modeling-lang.md](./modeling-lang.md), executed by the ordinary engine. The engine remains the single
semantic authority — shapes, port discipline, scope rules are checked there; the compiler adds name resolution,
modularity and source locations, and maps engine errors back to `file:line:col`. The statement layer is the
compiler's lowering target and the read surface for agents ([agent-interface](../agent-interface.md)), not an
editing surface: the source tree is the only persistence and the only way to change a model.

## Project layout

```
myproject/
  archi.toml              # manifest
  archi/
    src/
      messages.arch       # module `messages`
      conns.arch          # module `conns`
      auth.arch           # module `auth`
      auth_internals.arch # module `auth_internals`
      ui.arch             # module `ui`
      flows/
        login.arch        # module `flows.login`
```

The model lives under the tool's own `archi/` directory (alongside requirements, stress sessions and the
version archive), so it never collides with the host project's `src/`.

`archi.toml` marks the project root:

```toml
[project]
name = "myproject"
src = "archi/src"  # optional, default "archi/src"
preset = "default" # optional: "core", "default", or a relative path to a JSON preset file

[audit]                    # optional: settings for the link layer's tree scans
exclude = ["*.md", "docs/"] # scan-exclusion patterns: dir/ prefix, *.ext glob, exact path
```

The `[audit]` section is consumed by `archi`'s link layer ([code-link.md](../code-link.md#audit--dark-deltas-dark-spec));
the compiler validates its shape so a typo inside it is an `E_PROJECT`, not a silently ignored setting.

One file is one **module**; its module path is the dotted relative path under the source root
(`archi/src/auth/service.arch` is `auth.service`). Directory and file names must be identifiers. Discovery is a sorted
walk: module order — and everything downstream of it — is independent of filesystem iteration order.

The [preset](./ontology.md) is ambient: its names (`type_of`, and the ontology nodes of the `default` preset) are
visible in every module without import, and cannot be redefined.

## Modules, imports, visibility

A module's **exports** are its top-level definition names: root nodes (`def node X` with an undotted path), and its
`rel`, `conn` and `view` names. Nested nodes are not exported — they are reached by descent through their root.

```
import auth.service                       // every export of the module
import messages (LoginForm, AuthResponse) // just these
```

- Referencing another module's export requires importing it; the error names the module to import
  (`E_NOT_VISIBLE`).
- Imports are **visibility gates only**. The whole project compiles together in two passes, so import order carries
  no evaluation semantics and **import cycles are legal** — imports document dependencies, not load order.
- Importing a module that does not exist is `E_UNKNOWN_MODULE`; selectively importing a name the module does not
  export is an error.

**One definition site per name, project-wide**: two `def`s of the same absolute node path, of the same type name
(rel and conn share a namespace), or of the same view name are `E_REDECLARED`, with both sites reported — as is
capturing a preset name. Restating edges and applications is free (they are idempotent); `open` re-opens scopes
freely.

## Lexical structure

- UTF-8; identifiers `[A-Za-z_][A-Za-z0-9_]*`; paths join them with `.`.
- Comments run from `//` to end of line.
- One statement per line; no semicolons; no line continuations.
- **Blocks** are indentation (offside rule): a line ending in `:` opens a block; its items sit strictly deeper, at
  one common column; blank and comment-only lines are invisible; a dedent must return to an enclosing level. Tabs
  in indentation are rejected; indent with spaces.
- Reserved words, not usable as names: `import def open node view rel conn port trans in`. (A JSON-built model
  using one as a name cannot render to source; see [Fidelity](#fidelity).)

## Grammar

```ebnf
file         = { import_decl } { top_item }

import_decl  = "import" module_path [ "(" ident { "," ident } ")" ] NL
module_path  = ident { "." ident }

top_item     = def_view | def_rel | def_conn | def_node | open_block | edge_stmt | app_stmt

def_view     = "def" "view" ident NL
def_rel      = "def" "rel" [ "trans" ] ident ":=" slot_pat rel_arrow slot_pat NL
rel_arrow    = "->" | "<->"

def_conn     = "def" "conn" ident ":=" slot_pat lanes slot_pat NL
lanes        = "<->" [ slot_pat ]                        (* undirected; optional single carried slot *)
             | "->" [ slot_pat ] [ "," "<-" slot_pat ]   (* directed; forward and/or reverse carried slots *)

def_node     = "def" "node" path ( NL | ":" NL INDENT node_body DEDENT )
node_body    = { port_decl | def_node | open_block | edge_stmt | app_stmt }
port_decl    = "port" ident NL

open_block   = "open" path ":" NL INDENT open_body DEDENT
open_body    = { def_node | open_block | edge_stmt | app_stmt }   (* no port_decl *)

edge_stmt    = path type_ref path [ views ] NL           (* rel or conn edge, decided by the type's kind *)
type_ref     = ident [ "(" carrier_arg { "," carrier_arg } ")" ]
carrier_arg  = [ "->" | "<-" ] path                      (* a concrete carried node *)
views        = "in" ident { "," ident }

app_stmt     = path [ "(" bare_pat ")" ] "=" ident "." ident NL   (* outer[(route)] = Child.port *)

slot_pat     = "*" | path | "(" bare_pat ")"
bare_pat     = "*" | path | path ident "*"               (* any | exact | classified *)
path         = ident { "." ident }
```

The carrier-vs-target ambiguity after a lane arrow resolves by lookahead: a pattern followed by another pattern (or
by the `,` lane separator) was the lane's carried slot; a pattern followed by end of line was the target.

## Constructs

### Nodes and ports

```
def node AuthService:        // the node's interface: its declared ports, in one place
  port handle_login
  port handle_get_token
  port send_audit_log

def node LoginForm           // portless: no block needed
def node Orders.RefundHandler   // dotted form: augmentation into an existing scope
```

Ports are **declared at the node's definition** — the definition is the interface, and interface changes are diffs
on the defining file. Every port used by an edge or application must be declared on its node (`E_UNDECLARED_PORT`);
`open` blocks cannot declare ports. Lowering emits the declared ports on the node's `define` statement; a port's
connection type and side are still fixed by its first use, and a declared, never-wired port is the `unused_port`
finding, not an error. (The JSON API keeps creation-on-first-use for ports; declare-first is the source
discipline.)

### `open`: scope in another file

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

`open` re-opens an existing node's scope: definitions inside land in it, names inside resolve against it. Interface
and internals can live in different files — or the internals of one node can be split across several. Opens resolve
order-independently: an `open` waits until its target exists, wherever the defining module sorts.

### Relations

Unchanged from the [core spec](./modeling-lang.md#relation), minus semicolons:

```
def rel trans of_sort := * -> *
def rel has_pii := (Service type_of *) -> (Data type_of *)

Service type_of AuthService
Payments fails_via Orders in fault_prop
```

### Connections: lanes

A conn shape is `source LANES target`. Direction means **initiation**; carried slots ride on the lanes:

| form | meaning | statement fields |
|------|---------|------------------|
| `* -> *` | directed, no payload | — |
| `* ->P *` | directed, forward payload | `carrier` |
| `* ->P, <-Q *` | request/response | `carrier`, `rev_carrier` |
| `* ->, <-Q *` | pull: initiate forward, payload back | `rev_carrier` |
| `* <-> *` | undirected | — |
| `* <->P *` | undirected, single payload | `carrier` |

Carried slots are patterns: a bare path is an exact node, `*` is any, `(X rel *)` is classified. Illegal: a leading
or lone `<-` lane (flip the ends instead), and `<->` combined with a reverse lane. Port sides are unchanged — the
source port is `source` even when payload flows back.

```
def conn login := * ->LoginForm, <-AuthResponse *   // bidirectional request/response
def conn send  := * ->(Message type_of *) *         // classified payload
```

### Connection edges

`Node.port type Node.port` — a conn end's **last segment is the port**, the prefix is the node. Carried nodes go in
parens after the type name:

```
UI.login login AuthService.handle_login in login_flow   // carriers inferred (both lanes exact)
A.out send(OrderCreated) B.inbox                        // bare: binds the single carrying lane
A.req rpc(->Query, <-Result) B.serve                    // tagged: one per lane
```

Rules: a lane whose pattern is an **exact node may be omitted** — the compiler fills it in (lowered statements are
always fully explicit); `*`/classified lanes must be named (`E_CARRIER_REQUIRED` at compile, with the edge's span).
A bare argument is legal only when exactly one lane carries. Two arguments must tag their lanes.

### Applications

```
handle_login = LoginHandler.handle             // inside a block: bare port of the block's node
AuthService.handle_login = LoginHandler.handle // flat form, e.g. at top level
events(OrderCreated) = OrderHandler.handle     // routed by carried-node pattern
```

The left side names the delegating node's port (bare inside its block; `path.port` otherwise); the right side is
always `Child.port` — a **direct child**, per the core rule. The outer port must have a connection attached when
the application applies; lowering sequences applications by their delegation chains — the application that
attaches a port lowers before the applications delegating through it — so chains read outward-in wherever and in
whatever order they were authored.

### Views

```
def view login_flow
UI.login login AuthService.handle_login in login_flow, audit
```

## Name resolution

Every reference lowers to an absolute path; the statement layer keeps its "no ambient scope" contract. The lexical
rule for a path's **first segment**:

1. the innermost enclosing block's node's children — *semantic* children, wherever in the project they are defined;
2. enclosing blocks, outward;
3. file scope: the module's own top-level definitions, its imports, and the preset.

Later segments descend. Block children therefore shadow file-scope names; preset names are visible everywhere and
shadowable by children. Rel, conn and view names resolve in their flat namespaces against file scope (own ∪
imported ∪ preset).

An edge statement's kind follows its type name: a rel name reads its ends as whole node paths; a conn name splits
each end into node and port.

## Lowering and determinism

The project lowers to **one statement batch**, a function of the model alone — module names, file splits and
authoring order never move a statement:

1. nodes, parents before children (path order), each `define` carrying its declared ports;
2. views (name order);
3. rel types, topologically by pattern references between them (name tie-break) — a reference cycle is
   `E_DEF_CYCLE`;
4. conn types (name order);
5. rel edges, grouped by type in the same topological order — classifier edges land before the shapes that consult
   them — canonical surface order within a group;
6. conn edges, canonical surface order;
7. applications, delegation-chain order — an application lowers after the application that attaches its outer
   port — canonical surface order among the ready.

The batch executes on a fresh workspace holding the manifest's preset. Identical *models* produce an identical
batch, bit for bit — permuting file discovery changes nothing, and neither does renaming or splitting modules.
A statement→span table maps any engine rejection back to the source line, so shape violations, port conflicts
and scope errors read as ordinary compile errors.

## Errors

Compile diagnostics carry `file:line:col` and a stable code. New codes, on top of the
[statement catalog](./errors.md#catalog) (whose codes pass through, localized):

| code | raised when |
|------|-------------|
| E_PROJECT | the manifest, source tree or preset file cannot be read or parsed |
| E_UNKNOWN_MODULE | an `import` names no module of this project |
| E_NOT_VISIBLE | the name is defined in an unimported module; the message names it |
| E_UNDECLARED_PORT | an edge or application uses a port its node does not declare |
| E_DEF_CYCLE | rel-type shapes reference each other cyclically |

`E_PARSE` covers lexical and grammatical failures, located at the offending token. The `unused_port` finding
(declared, nothing attached) is the source-format counterpart of interface-first construction — reported by
`check`, never a rejection.

## Fidelity

Dumps render in the surface syntax: creation statements are valid source, so a dump pastes back into a module and
recompiles to the identical model. One inherent caveat: a statement-built model may use reserved words as names or
rely on use-created (undeclared) ports — such models replay via the statement layer but do not round-trip through
source. Reads have no surface form: the source is state, not a transcript.

## Worked example

`archi.toml`

```toml
[project]
name = "auth"
```

`archi/src/messages.arch`

```
def node LoginForm
def node AuthResponse
def node CredHash

Data type_of LoginForm // Data and Service come from the default preset
Data type_of AuthResponse
Data type_of CredHash
```

`archi/src/conns.arch`

```
import messages

// request/response: the forward lane carries LoginForm, the reverse lane
// carries AuthResponse back
def conn login := * ->LoginForm, <-AuthResponse *

// one-way, payload constrained by classification
def conn store := * ->(Data type_of *) *
```

`archi/src/auth.arch` — the interface in one place

```
def node AuthService:
  port handle_login
  port handle_get_token
  port send_audit_log

Service type_of AuthService
```

`archi/src/auth_internals.arch` — the inner structure in another

```
import auth
import conns
import messages

open AuthService:
  def node Storage:
    port save_cred_hash
    port purge_cred
  def node LoginHandler:
    port handle
    port persist

  LoginHandler.persist store(->CredHash) Storage.save_cred_hash
  handle_login = LoginHandler.handle // boundary port realized by an inner port
```

`archi/src/ui.arch`

```
import auth
import conns

def view login_flow

def node UI:
  port login

// both carriers inferred: the type's lanes are exact nodes
UI.login login AuthService.handle_login in login_flow
```

`archi check` compiles the project and reports findings (here: the declared-but-unwired ports, interface-first);
`archi nkp` analyzes the compiled model; `archi build --emit-batch -` shows the lowered statements. This project
lives as the fixture `crates/modeling-lang/tests/fixtures/auth`.
