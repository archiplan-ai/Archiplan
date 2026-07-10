# Code-link

During implementation, the thread between the typed spec and the files that realize it is
exactly what teams lose: months later nobody can say which code answers which element,
and every recovery attempt — commit mining, hand-maintained matrices — fights ambiguity
forever. A code-link records that some code realizes some spec element, machine-checkably:
captured where intent is known, verified against drift with an alarm that means
something, and aggregated into an audit that says what moved with no architectural
account. The early pin-only model made the link itself the thing that rotted — every edit
anywhere read as drift, verify cried wolf, and ritual re-pinning laundered the real
thing — so the record had to split into a fact that never moves and a projection that is
recomputed instead of maintained.
