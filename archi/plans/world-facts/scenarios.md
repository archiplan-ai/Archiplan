# Scenarios

- An operator mints a world fact, fills the prose, and check holds the file until every slot is written and every reference resolves. Runs on cargo test (world_e2e — no infrastructure).
- An operator asks what conditions a node and gets the covering facts back with their paths, without knowing any phrase from the fact. Runs on cargo test (world_e2e — no infrastructure).
- An agent asks the read envelope about an element and receives the conditions bearing on it beside the model slice. Runs on cargo test (read_e2e — no infrastructure).
- An operator retires a fact that a dependant, a link and an open plan all hold, and is handed the commands that clear each one in order. Runs on cargo test (world_e2e — no infrastructure).
- A plan closes on the scenarios of the facts covering its nodes, reports what moved since it was authored, and refuses the final latch while a scenario carries no link to code. Runs on cargo test (plan_e2e — no infrastructure).
- An operator searches by phrase, finds nothing, and is told the traversal that answers exactly. Runs on cargo test (search_e2e — no infrastructure).
- A project written before the wing upgrades, keeps a green check, and fills its wing by running the migration skill against its own intents and plan stories. Runs on cargo test (init_e2e — no infrastructure).
- A fact is written before the model reaches it, and check names the elements no recorded behavior arrives at without failing the tree. Runs on cargo test (check_e2e — no infrastructure).
