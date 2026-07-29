---
kind: functional
origin: intent
satisfied-by: [Updater]
deferred:
---

# check-update names the drift

`archi check-update` asks the server's `/version` and answers in one
line: up to date, or the newer number and the verb that fetches it. An
unreachable server is a named refusal, never a hang or a stack trace.

## System Context

The server publishes `{"latest": …}` at `/version`; the binary knows its
own number at compile time. The check is a pure read — it touches
neither the binary nor the spec, and it runs anywhere: no project, no
seat, no git required.

## Satisfy

`Updater.check` shells the request out and compares the two numbers;
the Cli routes the verb outside every guard.

- test — a local base URL serving a fixture `/version` answers all three
  ways: equal → up to date, newer → names it, dead → named refusal
