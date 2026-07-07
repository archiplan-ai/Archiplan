# Verification bullets silently don't count without the em-dash

**Kind:** friction (doc authoring) · found by the self-hosting bootstrap

A `Satisfy` bullet is recognized as a verification only as `- test — …` or `- type-level — …`
with a U+2014 em-dash (`schema::verification_bullet`,
`crates/archi/src/docs/schema.rs:521-529`). An ASCII hyphen, an en-dash, a `*` bullet or a tag
typo (`tests`) all silently fail the match: the requirement still compiles, but its verification
count is zero and the only signal is an `unverified_satisfaction` finding an author may not
connect to a punctuation choice.

## Impact

Agents and humans both reach for `-` before `—`. The failure is invisible at the file (the bullet
*looks* right), degrades scoring rather than erroring, and the finding's wording ("satisfaction
without verification") does not say *why* the bullets were not counted.

## Fix shape

Accept `-`/`–`/`—` after the tag, or keep the strict form but add a targeted diagnostic or
finding when a `Satisfy` bullet starts with `test`/`type-level` yet fails the full pattern —
"this bullet looks like a verification; use `- test — …`". A doc scaffolder (see
`no-init-or-doc-scaffolding.md`) would also erase the class.
