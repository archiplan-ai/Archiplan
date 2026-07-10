---
affects: [DocsCompiler]
outcome: breaking
---

# The hyphen voids the verification

A `Satisfy` bullet counts as a verification only as `- test — …` or `- type-level — …` with a
U+2014 em-dash. An author reaches for the ASCII hyphen, an en-dash, a `*` bullet, or types
`tests`, and the line still parses as ordinary prose: the requirement compiles, its verification
count is zero, and the only signal is an `unverified_satisfaction` finding the author has no
reason to trace back to a punctuation choice.

## Attractor

The failure is invisible at the file — the bullet *looks* right — and it degrades scoring rather
than erroring, so the ratchet quietly loosens: a satisfied requirement reads as verified-by-nothing
and the finding's wording never names the cause. The trap is live in this very model's doc schema
(`verification_bullet` matches U+2014 alone), and every agent and human reaches for `-` before `—`.
A tool whose whole purpose is an honest spec↔code record cannot afford a verification that voids
in silence.

## Resolution

Broke, as filed. Answered this round by making the recognizer refuse to fail silently: it accepts
`-`/`–`/`—` after the tag, and a bullet that opens with a verification tag yet still misses the
shape draws a located diagnostic that prints the canonical form rather than passing as prose. A
would-be verification can no longer void without a word. Derived: verifications-forgive-the-dash.
