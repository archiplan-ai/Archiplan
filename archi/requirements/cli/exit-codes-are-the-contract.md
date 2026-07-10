---
kind: non-functional
origin: intent
satisfied-by: [Cli]
deferred:
---

# Exit codes are the contract

Three exits, one meaning each: 0 — the compile is clean, findings if any are advisory,
and benign no-ops (an unchanged `version save`) are successes; 1 — the project fails to
compile, a doc violates its schema or the archive fails its seal, diagnostics printed as
`file:line:col: CODE: message`; 2 — the invocation itself is malformed, the usage text on
stderr. Asked-for help is output, not an error: `--help` and `--version` answer on
stdout, exit 0, needing no project anywhere near the working directory. Output is
human-readable one-liners by default — findings carrying their hint — and structured JSON
under `--json`, with `archi read` speaking the agent envelope verbatim as that contract's
zero-infrastructure transport.

## System Context

CI gates and agent loops branch on these codes blindly; an advisory state that flipped an
exit, or a usage error indistinguishable from a broken build, would gate the wrong
thing — `findings-never-block` is the advisory half of the same promise.

## Satisfy

`Cli` (the exit discipline across every verb; the meta flags answered before project
location; human and JSON renderings split per flag).

- test — cli_e2e::help_and_version_answer_on_stdout_without_a_project
- test — cli_e2e::the_malformed_invocation_keeps_its_contract
- test — check_e2e::errors_withhold_the_read
- test — search_e2e::advisory_states_search_fine_and_a_save_still_reports_unchanged
