---
kind: functional
origin: intent
satisfied-by: [Updater]
deferred:
---

# check-update names the drift

`archi check-update` asks the release feed for the latest tag and
answers in one line: up to date, or the newer number and the verb that
fetches it. An unreachable feed is a named refusal, never a hang or a
stack trace.

## System Context

The feed is the repository's GitHub releases — the `releases/latest`
redirect names the tag, the API is the fallback; the binary knows its
own number at compile time. The check is a pure read — it touches
neither the binary nor the spec, and it runs anywhere: no project, no
seat, no git required.

## Satisfy

`Updater.check` shells the request out and compares the two numbers;
the Cli routes the verb outside every guard.

- test — a local feed base serving a fixture `latest` answers all three
  ways: equal → up to date, newer → names it, dead → named refusal
