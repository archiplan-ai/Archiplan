# Error Contract

The write API is built for agents: failures are machine-readable, statements are atomic, and identical retries are
always safe.

## Statement results

Every statement yields exactly one of:

- **applied** — the model changed.
- **noop** — the statement restates something identical that already exists (`node Service` twice, restating an
  existing edge, redeclaring a type with the same definition). Replays and at-least-once retries are safe.
- **error** — the statement was rejected; the model is untouched.

Interactive statements apply one at a time. A batch submitted as one request is atomic: any error rolls the whole
batch back and reports the failing statement's index alongside the error.

`delete` results additionally carry the cascade set — everything removed, rendered as statements.

## Error shape

| field | meaning |
|-------|---------|
| code | stable identifier from the catalog below |
| message | human-readable one-liner |
| subject | the offending statement, as submitted |
| refs | paths/ids of the elements involved |
| expected / actual | the violated constraint, where applicable (pattern vs node, type vs type) |
| hint | suggested next step, phrased as a runnable statement or query (e.g. `ports Orders`) |

## Catalog

| code | raised when |
|------|-------------|
| E_PARSE | the statement does not parse; position and expected tokens included |
| E_UNKNOWN_NAME | a referenced node / type / view / path does not resolve in scope; `kind` says which |
| E_DUP_NAME | node creation or rename collides with a sibling's name |
| E_REDECLARED | a rel / conn / view is redeclared with a different definition; the existing definition is included |
| E_SHAPE_VIOLATION | an end or carrier fails the type's pattern at edge creation; slot, pattern and node included |
| E_CARRIER_REQUIRED | a ternary connection is instantiated without a carrier |
| E_CARRIER_FORBIDDEN | a binary connection is instantiated with a carrier |
| E_PORT_TYPE_CONFLICT | a port is reused with a different connection type than its first use fixed |
| E_PORT_SIDE_CONFLICT | a port of a directed type is reused on the opposite side |
| E_NO_OUTER_PORT | an application delegates a port no connection attaches to |
| E_AMBIGUOUS_DELEGATION | two qualified delegations on one port match the same carried node |
| E_STDLIB_PROTECTED | attempt to delete or divergently redeclare a stdlib element |

Codes are append-only: new codes may appear; existing codes never change meaning.

## Errors vs findings

Errors reject writes. States that are legal mid-construction but suspect — an edge whose conformance drifted after a
classifier edge was removed, carried traffic matching no delegation, a delegated port with no attached connections, a
view with no edges, a type with no instances — are **findings**: surfaced by check/query operations
([queries](./queries.md)), never by rejecting writes.
