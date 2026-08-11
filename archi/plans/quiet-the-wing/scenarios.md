# Scenarios

- An operator writes one honest world fact and reads a check whose coverage report is short enough to act on. Runs on cargo test (check_e2e — no infrastructure).
- An operator declares an element internal with a reason, and the coverage report goes quiet for it while a renamed element breaks the declaration loudly. Runs on cargo test (check_e2e — no infrastructure).
- An operator changes the code under an unchanged scenario and the link fails, naming the side that moved. Runs on cargo test (link tests — no infrastructure).
- An operator reruns a mint on a skeleton nobody has written into and the verb converges instead of refusing. Runs on cargo test (world_e2e — no infrastructure).
- A plan on a tree that holds no world facts closes without a refusal, and the same plan on a tree that has a wing refuses until a fact covers one of its nodes. Runs on cargo test (plan_e2e — no infrastructure).
- An operator upgrades and reads a briefing that says only what no command prints. Runs on cargo test (init_e2e — no infrastructure).
