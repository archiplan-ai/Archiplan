# Requirements

Requirement is a unit of desirable shape of software, a claim the architecture must uphold.

## Requirement Body

Requries at least a prose description.

### System Context

Pre-existing landscape a system architecture should land onto: external services, specific tech etc.

## Requirement kinds

Functional or Non-Functional

## Unchecked requirements

Some requirements may be acknowledged as a reaction to a stressor, but are not neccessary to be address in the current architecture. Such requirements must be uncked manually by the user, so Archiplan does not emit errors on unsatisfied requirements.

## Fusion

Junction of serveral requirements often result in new requirements. Archiplan keeps track of these relations of form `req1 * req2 * reqN => req'`

## Origin

It's important to track where each requirement came from:
- Initial (not derived from anything, cannot be added during stress-session)
- Subrequirement (derived from parent requirement)
- Fused from Set(Requirement) (derived from intersection of requirements, see #Fusion)
- Stressor/ Set(Stressor) (derived as a solution to [stressor(s)](stressors.md))

## Satisfy

A claim that a certain element of the spec satisfies the requirement. Equipped with a prose 
explanation and an optional set of verifications.

### Transitivity

If a parent requirement is satisfied, subrequirements are satisfied by definition.

### Verification

Defines how the fact that a certain element of the spec satisfies a requirement can be proved.

Varinants:
- Test - describes a test(s) that performs nessessary checks
- TypeLevel - describes how the requirement can be enforced at type level

#### Verification * Scoring

Verifications are subject to scoring, the more requirements have formulated verifications the better.

## Requirements * Agent Interface

- **Add a requirement** with id, description, and an origin.
- **Remove a requirement** (and its satisfaction, if any).
- **Satisfy** a requirement with a prose explanation of *how* the
  current design meets it (**inline** at the requirement: one satisfaction
  record per requirement).
- **Unsatisfy** a requirement when the justification no longer holds.
- **List requirements** with satisfaction status and origin.

## Requirements * Human Interface

Each requirement must have a slug -- a short form handle to be used to reference the requirement from other places like stress-session tables.

## Stored as files

Requirements are stored as structured .md files under `archi/`, subrequirements live in the parent's files. If parent requirements is too large it can be declared as a folder, `requirement.md` describes epic parent requirement, subrequirements are listed as individual files under the folder

File structure:
- Name
- Slug (auto-derived)
- Body
  - System Context
- Origin
- Satisfy