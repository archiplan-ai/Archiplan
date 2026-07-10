---
kind: functional
origin: intent
satisfied-by: [Compiler]
deferred:
---

# The manifest marks the root

`archi.toml` marks the project root and is the whole configuration surface, and it
rejects what it does not understand: unknown fields anywhere in it are `E_PROJECT`,
never silently ignored settings. Below the root, discovery is a sorted walk of the
source directory — one file is one module, its module path the dotted relative path
under the source root, and module order (and everything downstream of it) is
independent of filesystem iteration order.

The model lives under the tool's own `archi/` directory — alongside requirements,
stress sessions and the version archive — so it never collides with the host
project's `src/`:

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

The manifest's sections:

```toml
[project]
name = "myproject"
src = "archi/src"  # optional, default "archi/src"
preset = "default" # optional: "core", "default", or a relative path to a JSON preset file

[audit]                     # optional: settings for the link layer's tree scans
exclude = ["*.md", "docs/"] # scan-exclusion patterns: dir/ prefix, *.ext glob, exact path
```

The `[audit]` section is consumed by the link layer's tree scans (`dark-deltas-are-code`),
not by the compiler — but the compiler validates its shape, so a typo inside it is an
`E_PROJECT`, not a setting that silently stops excluding. `E_PROJECT` likewise covers a
manifest, source tree or preset file that cannot be read or parsed at all. The multi-repo
member declarations ride the same manifest and the same strictness (`members-are-declared`).

A file's module path is derived mechanically: `archi/src/auth/service.arch` is
`auth.service`. Directory and file names must be identifiers, so every module path is a
valid dotted path. The manifest's `preset` choice loads the ambient stdlib whose names
are visible in every module without import (`names-resolve-from-the-inside-out`).

## System Context

The `.arch` tree is the only persistence of the model (`source-is-the-only-truth`); every
verb starts by finding the root and compiling the project fresh. A configuration key that
is misspelled and silently ignored is worse than a rejection — the author believes a
setting is in force when it is not.

## Satisfy

`Compiler` (its `load_sources` door pulls the manifest and the `.arch` module texts from
the tree; manifest parsing denies unknown fields and surfaces every failure as a located
`E_PROJECT`; discovery sorts the walk so downstream order never depends on the
filesystem).

- test — project::manifest_accepts_a_valid_audit_section
- test — project::a_typo_inside_audit_is_loud, project::a_non_list_exclude_is_loud
- test — source_e2e::compilation_is_deterministic_under_source_order
