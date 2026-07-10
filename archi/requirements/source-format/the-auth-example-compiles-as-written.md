---
kind: non-functional
origin: intent
satisfied-by: [Compiler, Engine]
deferred:
---

# The auth example compiles as written

The worked example is executable spec: the auth project — `messages`, `conns`, `auth`,
`auth_internals` and `ui`, one module each for the data, the connection types, the
interface, the internals and the client — lives as a fixture and compiles exactly as the
docs print it. The interface/internals split across `auth.arch` and `auth_internals.arch`,
the request/response `login` conn with inferred carriers and the `handle_login`
delegation are demonstrations that cannot drift from the language they demonstrate: the
fixture answers queries, passes `check`, and survives NKP.

## System Context

Every construct card in this intent quotes the auth example; a quoted example that
stopped compiling would be spec rot of exactly the kind archi exists to catch. The
fixture is the compile-checked twin of the prose — `docs-compile-with-the-model` guards
the machine fields, this guards the narrative.

## Satisfy

`Compiler` (compiles the fixture project through the full pipeline). `Engine` (executes
its batch; the compiled model answers the documented queries).

- test — source_e2e::the_auth_fixture_compiles_and_answers_queries
- test — source_e2e::the_fixture_passes_nkp_and_check
- test — declared_ports::declared_ports_exist_before_any_edge
