# Permission layer

Every call is authorized here before it reaches the Store. The **Guard** on each
entity — built by the entity utility, so every exposed entity carries one —
proxies into **PermManager**, the authorization engine. Grants attach directly to
accounts. The access data lives in the store
([core-data-model.md](core-data-model.md)).

## PermManager — the engine

PermManager is the policy the Guard calls. It ships in the **allow** core crate
and reads the access data through the **Store Decorator**
([store-layer.md](store-layer.md)).

- the **policy contract** — given a **principal** and the call's
  `(project, entity type, op)`, it returns a decision; it decides on these inputs
  alone, whoever produced them;
- the **baseline policy** — the flat `(account, project, entity type) → R / RW`
  allow-model, one implementation of the contract. It reads the **token's grants
  by `tokenId`** (a subset of the account's —
  [core-data-model.md](core-data-model.md)), matches the call's namespace and
  project, and decides `grants[type] ≥ op`;
- a **granular policy** — a **deferred** extension: another implementation of the
  same contract, plugged in when a concrete case needs finer control, while the
  core stays fixed.

## The access model

Access is the **sum of grants**: you have exactly what was granted. The unit is
the **entity type inside a project**.

### Namespace — owner-only operations

| Operation | Who |
| --- | --- |
| Change namespace (parameters) | **owner only** |
| Delete namespace | **owner only** |
| Grant / revoke namespace management | **owner only** |

### Namespace management — the single grantable namespace right

Granted by the owner. Covers:

- add / remove members in the namespace;
- create / delete projects;
- grant / revoke access to entity types within projects.

### A project is a container of entity types

A project holds heterogeneous, client-defined entity types. Access is granted
**per entity type**.

### Access grant — per (account, project, entity type)

The unit is `(account, project, entity type) → R | RW`:

| Level | Meaning |
| --- | --- |
| `R` | read that entity type in that project |
| `RW` | read and write that entity type |

A flat **set of types** per project: you see and touch exactly the types granted
to you; to give access to a type, grant it. Granted by the owner or namespace
management. This is **type-level** ("all of a type"); instance-level stays out of
scope until a concrete case needs it.

## Rules

1. **Hierarchy: owner → management → member.** Managers act on regular members;
   the owner acts on managers.
2. **The subject is the account.** A call carries the account's authority via its
   token, which holds a subset of the account's grants
   ([core-data-model.md](core-data-model.md)).
3. **Leaving is allowed** — removing yourself touches your own membership; the
   owner stays.
4. **Visibility = access, per entity type.** You see exactly the types granted to
   you. Namespace membership reveals the namespace; a grant reveals its entity
   type.
5. **Owner rights are implicit.** The owner holds every right in their namespace
   by virtue of `owner_id` alone; PermManager derives it in one check. The grant
   table holds members' explicit grants; the owner's authority lives in
   `owner_id`, always present.
6. **Every right is an explicit grant.** A new member begins empty and gains a
   type by being granted it.
