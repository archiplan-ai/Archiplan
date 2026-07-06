# Error Contract

The write API is built for agents: failures are machine-readable, batches are atomic, and identical retries are
always safe.

## Statement results

Every statement in a batch yields exactly one of:

- **applied** — the model changed.
- **noop** — the statement restates something identical that already exists (a `define` of an element already
  defined that way, a `redefine` that changes nothing, restating an existing edge). Replays and at-least-once
  retries are safe.
- **error** — the statement was rejected; the model is untouched.

A batch is atomic: any error rolls the whole batch back and reports the failing statement's index alongside the
error. Read statements yield their result payload (a graph or findings) in place.

`delete` results — and node `redefine` results — additionally carry the cascade set: everything removed, rendered
as statements.

## Error shape

| field | meaning |
|-------|---------|
| code | stable identifier from the catalog below |
| message | human-readable one-liner |
| subject | the offending statement, as submitted |
| refs | paths/ids of the elements involved |
| expected / actual | the violated constraint, where applicable (pattern vs node, type vs type) |
| hint | suggested next step, phrased as a runnable statement (e.g. a `query` scoped to the involved node) |

## Catalog

| code | raised when |
|------|-------------|
| E_PARSE | the statement is not a well-formed statement object: unknown `stmt`, missing or ill-typed field, malformed path; the offending field is named |
| E_UNKNOWN_NAME | a referenced node / type / view / path does not resolve — including `redefine` of an element that does not exist; `kind` says which |
| E_DUP_NAME | a `rename` collides with a sibling's name |
| E_REDECLARED | a `define` differs from the existing definition of the name — including a rel / conn kind mismatch; the existing definition is included |
| E_SHAPE_VIOLATION | an end or carried node fails the type's pattern at edge creation; slot (`source`, `target`, `carrier`, `rev_carrier`), pattern and node included |
| E_CARRIER_REQUIRED | a lane with a carried slot is instantiated without naming its carried node; the lane is named |
| E_CARRIER_FORBIDDEN | a carried node is named on a lane without a carried slot; the lane is named |
| E_PORT_TYPE_CONFLICT | a port is reused with a different connection type than its first use fixed |
| E_PORT_SIDE_CONFLICT | a port of a directed type is reused on the opposite side |
| E_NO_OUTER_PORT | an application delegates a port no connection attaches to |
| E_AMBIGUOUS_DELEGATION | two qualified delegations on one port match the same carried node |
| E_CROSS_SCOPE | a connection joins nodes of different scopes, or an application's inner node is not a direct child of the delegating node |
| E_STDLIB_PROTECTED | attempt to delete, rename or divergently redefine a stdlib (preset) element, or to tag/untag a stdlib edge — tags on it would not survive a dump replay |
| E_PRESET_INVALID | a preset does not load: a non-creation statement, a rejected statement, or a missing/divergent `type_of` classifier ([ontology](./ontology.md)) |

Codes are append-only: new codes may appear; existing codes never change meaning.

The [source format](./source-format.md#errors) adds compile-time codes — `E_PROJECT`, `E_UNKNOWN_MODULE`,
`E_NOT_VISIBLE`, `E_UNDECLARED_PORT`, `E_DEF_CYCLE` — and localizes every statement-level code above to
`file:line:col` via the compiler's span table.

## Errors vs findings

Errors reject writes. States that are legal mid-construction but suspect — an edge whose conformance drifted after a
classifier edge was removed or a type was redefined, carried traffic matching no delegation, a delegated port with no
attached connections, a declared port with no attachments at all (`unused_port`), a view with no edges, a type with
no instances — are **findings**: surfaced by check/query operations ([queries](./queries.md)), never by rejecting
writes.
