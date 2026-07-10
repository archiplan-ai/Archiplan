# CLI

Everything reaches archi through one thin runner: humans, agents and CI invoke verbs that
compile the project fresh and render what the subsystems answer. A runner that added its
own vocabulary would fork the semantics; one that guessed at its project, swallowed
distinct failure modes into one exit code, or printed for humans when a machine is
reading would break every script and repair loop stacked on top of it. The CLI's job is
to be predictable plumbing: locate, compile, delegate, render, exit honestly.
