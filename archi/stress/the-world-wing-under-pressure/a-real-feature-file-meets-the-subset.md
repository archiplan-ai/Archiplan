---
affects: [Gherkin, DocsCompiler]
outcome: breaking
---

# A real feature file meets the subset

Paste a working feature file from a live project into `## Scenarios`. It carries a
`Background:`, a `Scenario Outline:` with its `Examples:` table, a `@tag` line above
the feature and a docstring inside a step. The grammar knows six keywords. The parse
raises `E_DOC` at the third line, and `check` blocks the whole tree over a scenario
that a real runner executes today.

## Attractor

The operator meets the wall once and routes around it. The scenarios move out of the
wing into the code tree where the runner already reads them, and the world fact keeps
a prose sentence in their place. The wing becomes a folder people copy out of instead
of into, and the executable half of the unit — the reason the fact and the scenario
were merged into one object — is gone.

## Resolution

The subset goes and the grammar reads the whole language, so a feature file that runs
today parses here unchanged: derived `the-grammar-takes-the-whole-language`, and
`scenarios-parse-or-the-check-fails` narrows to the blocking and the location of the
failure. The price is a real parser to keep correct, taken deliberately and recorded
in `the-scenario-travels-unedited`.
