---
kind: functional
origin: intent
satisfied-by: [Compiler.Resolver]
deferred:
---

# Imports are visibility gates

An import grants a module the right to see another module's exports — and nothing
else. The whole project compiles together in two passes, so import order carries no
evaluation semantics and import cycles are legal: imports document dependencies, not
load order. Referencing another module's export without importing it is
`E_NOT_VISIBLE`, and the message names the module to import; importing a module
that does not exist is `E_UNKNOWN_MODULE`; selectively importing a name the module
does not export is an error at the import site.

A module's exports are its top-level definition names: root nodes (`def node X`
with an undotted path) and its `rel`, `conn` and `view` names. Nested nodes are not
exported — they are reached by descent through their root. The two import forms:

```
import auth.service                       // every export of the module
import messages (LoginForm, AuthResponse) // just these
```

One definition site per name, project-wide: two `def`s of the same absolute node
path, of the same type name (rel and conn share a namespace), or of the same view
name are `E_REDECLARED`, with both sites reported — as is capturing a preset name.
Restating edges and applications is free (they are idempotent), and `open` re-opens
scopes freely (`open-reopens-a-scope`) — redeclaration polices definitions, not
restatements.

## System Context

Modules compile in sorted-name order and the format promises that file organization
carries no evaluation semantics (`uses-see-every-def`): a visibility gate is the
only meaning an import can safely have, because any load-order meaning would make
renaming a file a semantic change.

## Satisfy

`Compiler.Resolver` (binds cross-module references only through the importing
module's gate; its `E_NOT_VISIBLE` diagnostic carries the "import it" hint; the
def-collection pass reports duplicate definition sites with both spans).

- test — resolve::cross_file_references_require_imports
- test — resolve::selective_imports_gate_the_rest
- test — resolve::unknown_modules_and_exports_are_reported
- test — resolve::import_cycles_are_legal
- test — resolve::one_definition_site_per_name
