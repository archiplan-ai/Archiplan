# Requirements

Requirement is a unit of desirable shape of software, a claim the architecture must uphold.

## Requirement kinds

May be Functional or Non-Functional

## Unchecked requirements

Some requirements may be acknowledged as a reaction to a stressor, but are not neccessary to be address in the current architecture. Such requirements must be uncked manually by the user, so Archiplan does not emit errors on unsatisfied requirements.

## Satisfy

A claim that a certain element of the spec satisfies the requirement. Equipped with a prose 
explanation and an optional set of verifications.

### Verification

Defines how the fact that a certain element of the spec satisfies a requirement can be proved.

Varinants:
- Test - describes a test(s) that performs nessessary checks
- TypeLevel - describes how the requirement can be enforced at type level

#### Verification * Scoring

Verifications are subject to scoring, the more requirements have formulated verifications the better.

## Links

Requirement can contain links to arbitrary objects in [knowledge base](kb.md)

## Fusion

Junction of serveral requirements often result in new requirements. Archiplan keeps track of these relations of form `req1 * req2 * reqN => req'`

## Origin

It's important to track where each requirement came from:
- Initial (not derived from anything, cannot be added during stress-session)
- Subrequirement (derived from parent requirement)
- Fused from Set(Requirement) (derived from intersection of requirements, see #Fusion)
- Stressor/ Set(Stressor) (derived as a solution to [stressor(s)](stressors.md))
- Derived from a [system context](sys-context.md) node (e.g. "must integrate with payment gateway X")
- Derived from an [intent](intents.md)

## Capabilities

- **Add a requirement** with id, description, and an origin.
- **Remove a requirement** (and its satisfaction, if any).
- **Satisfy** a requirement with a prose explanation of *how* the
  current design meets it (**inline** at the requirement: one satisfaction
  record per requirement).
- **Unsatisfy** a requirement when the justification no longer holds.
- **List requirements** with satisfaction status and origin.

## Requirements * Human Interface

Each requirement must have a slug -- a short form handle to be used to reference the requirement from other places like stress-session tables.