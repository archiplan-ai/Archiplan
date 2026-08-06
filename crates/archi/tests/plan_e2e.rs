//! End to end through the real binary: a plan is a folder of markdown
//! records with lifecycle in `state.json` — minted by commands, authored by
//! editing the files — and waves gate on captured-then-asserted
//! code-links (`archi/requirements/planning/`,
//! `archi/requirements/planning/a-plan-is-a-folder-of-records.md`,
//! `archi/requirements/self-hosting/capture-at-the-join.md`). The legacy
//! `plan.json` form stays readable forever, read-only.

mod util;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use serde_json::Value;

static NEXT: AtomicUsize = AtomicUsize::new(0);

const MODEL: &str = "def conn wire := * -> *\n\
                     def node Gate:\n  port out\n\
                     def node Auth:\n  port inn\n  port creds\n\
                     def node Store:\n  port inn\n\
                     Gate.out wire Auth.inn\n\
                     Auth.creds wire Store.inn\n\
                     Service type_of Auth\n";

fn temp_project() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "archi-plan-e2e-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::SeqCst)
    ));
    fs::create_dir_all(dir.join("archi/src")).unwrap();
    fs::create_dir_all(dir.join("code")).unwrap();
    fs::write(
        dir.join("archi.toml"),
        "[project]\nname = \"t\"\npreset = \"default\"\n",
    )
    .unwrap();
    fs::write(dir.join("archi/src/model.arch"), MODEL).unwrap();
    fs::write(
        dir.join("code/store.rs"),
        "pub struct Store;\nimpl Store {\n    pub fn put(&mut self) {}\n}\n",
    )
    .unwrap();
    fs::write(dir.join("code/auth.rs"), "pub fn login() -> bool { true }\n").unwrap();
    put_requirement(&dir, "store-encrypted", "Store encrypted", "Store");
    put_requirement(&dir, "service-hardening", "Service hardening", "Service");
    util::worktree(&dir)
}

fn put_requirement(root: &Path, slug: &str, name: &str, satisfied_by: &str) {
    let dir = root.join("archi/requirements/hardening");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("hardening.md"), "# Hardening\n\nThe area.\n").unwrap();
    fs::write(
        dir.join(format!("{slug}.md")),
        format!(
            "---\nkind: functional\norigin: intent\nsatisfied-by: [{satisfied_by}]\ndeferred:\n---\n\n\
             # {name}\n\nSummary paragraph.\n\n## System Context\n\n## Satisfy\n\n\
             Prose claim.\n\n- test — proof sketch\n"
        ),
    )
    .unwrap();
}

/// Run the binary; return (success, stdout, stderr).
fn run(root: &Path, args: &[&str]) -> (bool, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_archi"))
        .args(args)
        .args(["--project", root.to_str().unwrap()])
        .output()
        .expect("archi runs");
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

fn ok(root: &Path, args: &[&str]) -> String {
    let (success, stdout, stderr) = run(root, args);
    assert!(success, "archi {args:?} failed:\n{stdout}\n{stderr}");
    stdout
}

fn fails(root: &Path, args: &[&str]) -> (String, String) {
    let (success, stdout, stderr) = run(root, args);
    assert!(!success, "archi {args:?} unexpectedly passed:\n{stdout}");
    (stdout, stderr)
}

/// A command cut from the surface is an unknown subverb: the plan usage
/// error, exit 2 — no tombstones.
fn usage_error(root: &Path, args: &[&str]) {
    let out = Command::new(env!("CARGO_BIN_EXE_archi"))
        .args(args)
        .args(["--project", root.to_str().unwrap()])
        .output()
        .expect("archi runs");
    let err = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(2), "archi {args:?}:\n{err}");
    assert!(err.contains("`plan` takes:"), "{args:?}: {err}");
}

/// Run the binary with text piped to stdin; return (success, stdout, stderr).
fn run_stdin(root: &Path, args: &[&str], stdin: &str) -> (bool, String, String) {
    use std::io::Write;
    use std::process::Stdio;
    let mut child = Command::new(env!("CARGO_BIN_EXE_archi"))
        .args(args)
        .args(["--project", root.to_str().unwrap()])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("archi runs");
    child.stdin.as_mut().unwrap().write_all(stdin.as_bytes()).unwrap();
    let out = child.wait_with_output().unwrap();
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// The lifecycle file of a record plan, parsed.
fn state_json(root: &Path, name: &str) -> Value {
    let text = fs::read_to_string(root.join(format!("archi/plans/{name}/state.json"))).unwrap();
    serde_json::from_str(&text).unwrap()
}

/// Author a record file: the test's stand-in for the human editor.
fn write_record(root: &Path, rel: &str, text: &str) {
    fs::write(root.join(rel), text).unwrap();
}

/// The `captured lNNNN …` ids of a `plan next` transcript.
fn captured_ids(stdout: &str) -> Vec<String> {
    stdout
        .lines()
        .filter_map(|l| l.strip_prefix("captured "))
        .filter_map(|l| l.split_whitespace().next())
        .map(str::to_string)
        .collect()
}

#[test]
fn the_record_folder_authors_by_editing_files() {
    let root = temp_project();
    ok(&root, &["version", "save", "-m", "first"]);

    // `use` mints the folder: charter and scenarios skeletons, lifecycle
    // in state.json — no plan.json is ever born again.
    let out = ok(&root, &["plan", "use", "mvp"]);
    assert!(out.contains("created plan `mvp` @ v0001"), "{out}");
    let dir = root.join("archi/plans/mvp");
    assert_eq!(
        fs::read_to_string(dir.join("mvp.md")).unwrap(),
        "# mvp\n\n## Stack\n\n## Architecture\n"
    );
    assert_eq!(fs::read_to_string(dir.join("scenarios.md")).unwrap(), "# Scenarios\n");
    assert_eq!(state_json(&root, "mvp")["state"], "draft");
    assert!(!dir.join("plan.json").exists());

    // The old authoring commands are gone from the surface: a dead command
    // falls through to usage, exit 2 — content is the files, edited in
    // place.
    for dead in [
        vec!["plan", "problem", "a tiny hardened store"],
        vec!["plan", "tech", "add", "Rust"],
        vec!["plan", "task", "desc", "t1", "persist rows"],
        vec!["plan", "scenarios", "add", "a user stores a row"],
    ] {
        usage_error(&root, &dead);
    }

    // Tasks mint as seeded skeleton files; a byte-equal re-mint
    // converges, an edited file refuses.
    let out = ok(&root, &["plan", "task", "add", "Store"]);
    assert!(out.contains("t1 Store"), "{out}");
    let out = ok(&root, &["plan", "task", "add", "Auth"]);
    assert!(out.contains("t2 Auth"), "{out}");
    assert!(dir.join("t1-store.md").exists() && dir.join("t2-auth.md").exists());
    let out = ok(&root, &["plan", "task", "add", "Store"]);
    assert!(out.contains("already minted"), "{out}");
    assert!(out.contains("t1-store.md stands"), "{out}");

    // Author the plan by editing its files — the whole old command surface
    // is a text editor now.
    write_record(
        &root,
        "archi/plans/mvp/mvp.md",
        "# mvp\n\n\
         a tiny hardened store\n\n\
         ## Stack\n\n\
         - Rust — user choice\n\n\
         ## Architecture\n\n\
         - `Store` — keeps the rows\n\
         - `Auth` — guards the door\n\
         - `Gate` — fronts the world\n\
         - `Store` realizes Rust\n\
         - `Auth` realizes Rust\n\
         - `Gate` realizes Rust\n",
    );
    write_record(
        &root,
        "archi/plans/mvp/scenarios.md",
        "# Scenarios\n\n- a user stores a row\n",
    );
    write_record(
        &root,
        "archi/plans/mvp/t1-store.md",
        "---\n\
         node: Store\n\
         owns: [store-encrypted]\n\
         ---\n\n\
         # t1 — Store\n\n\
         persist rows\n\n\
         ## Spec\n\n\
         - `Store`\n\
         - `Auth.creds wire Store.inn`\n\n\
         ## Inputs\n\n\
         ## Outputs\n\n\
         - code/store.rs\n\n\
         ## Stack\n\n\
         - sqlite via rusqlite\n\n\
         ## Verifications\n\n\
         ### store-encrypted\n\n\
         - test — rows encrypted at rest\n",
    );
    write_record(
        &root,
        "archi/plans/mvp/t2-auth.md",
        "---\n\
         node: Auth\n\
         owns: [service-hardening]\n\
         ---\n\n\
         # t2 — Auth\n\n\
         guard the door\n\n\
         ## Spec\n\n\
         - `Auth`\n\
         - `Gate.out wire Auth.inn`\n\
         - `Service type_of Auth`\n\n\
         ## Inputs\n\n\
         - from t1 — the store api\n\n\
         ## Outputs\n\n\
         - code/auth.rs\n\n\
         ## Stack\n\n\
         ## Verifications\n\n\
         ### service-hardening\n\n\
         - test — login hardened\n",
    );

    // A file past its skeleton is the author's — not re-mintable.
    let (_, err) = fails(&root, &["plan", "task", "add", "Store"]);
    assert!(err.contains("moved past its skeleton"), "{err}");

    // `task rm` unmints a leaf — the file is gone; a producer some task
    // inputs is held in place, the dependents named.
    ok(&root, &["plan", "task", "add", "Gate"]);
    assert!(dir.join("t3-gate.md").exists());
    let out = ok(&root, &["plan", "task", "rm", "t3"]);
    assert!(out.contains("removed"), "{out}");
    assert!(!dir.join("t3-gate.md").exists());
    let (_, err) = fails(&root, &["plan", "task", "rm", "t1"]);
    assert!(err.contains("feeds t2"), "{err}");

    // The read surfaces serve the files verbatim.
    let show = ok(&root, &["plan", "show"]);
    assert!(show.contains("problem: a tiny hardened store"), "{show}");
    assert!(show.contains("stack: Rust — user choice"), "{show}");
    assert!(show.contains("summary: Store — keeps the rows"), "{show}");
    assert!(show.contains("mapping: Rust realizes Gate"), "{show}");
    assert!(ok(&root, &["plan", "scenarios", "list"]).contains("1. a user stores a row"));
    let brief = ok(&root, &["plan", "task", "show", "t1"]);
    assert!(brief.contains("t1 Store — persist rows"), "{brief}");
    assert!(brief.contains("sqlite via rusqlite"), "{brief}");
    assert!(brief.contains("output: code/store.rs"), "{brief}");
    assert!(brief.contains("store-encrypted (owned"), "{brief}");
    assert!(brief.contains("verify: test — rows encrypted at rest"), "{brief}");
    assert!(ok(&root, &["plan", "list"]).contains("mvp @ v0001 (draft)"));
    assert!(ok(&root, &["plan", "status"]).contains("plan `mvp` @ v0001 (draft), 0 waves closed"));

    // The authored files verify and start; structure is frozen past draft.
    ok(&root, &["plan", "verify"]);
    let out = ok(&root, &["plan", "start"]);
    assert!(out.contains("wave 1 in flight: t1"), "{out}");
    assert!(ok(&root, &["plan", "status"]).contains("(started)"));
    assert_eq!(state_json(&root, "mvp")["state"], "started");
    let (_, err) = fails(&root, &["plan", "task", "add", "Gate"]);
    assert!(err.contains("tasks are cut in draft"), "{err}");
    let (_, err) = fails(&root, &["plan", "task", "rm", "t2"]);
    assert!(err.contains("past draft"), "{err}");
    assert!(err.contains("plan reset"), "{err}");

    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn batch_runs_the_mint_verbs_and_stops_at_the_first_failure() {
    let root = temp_project();
    ok(&root, &["version", "save", "-m", "first"]);

    // One invocation mints the plan and its tasks: comments and blank
    // lines are skipped, CRLF tolerated, `--project` is forwarded to
    // every line. The failing mint stops the batch exactly there.
    let script = "# the record plan in one call\r\n\
                  plan use mvp\n\
                  plan task add Store\n\
                  \n\
                  plan task add Auth\n\
                  plan task add Nope\n\
                  plan task add Gate\n";
    let (success, out, err) = run_stdin(&root, &["batch"], script);
    assert!(!success, "{out}");
    assert!(err.contains("batch stopped at line 6"), "{err}");
    assert!(err.contains("E_MODEL_REF"), "{err}");
    assert!(out.contains("[3] plan task add Auth"), "the lines before it ran: {out}");

    // Everything before the stop is applied; nothing after existed.
    let dir = root.join("archi/plans/mvp");
    assert!(dir.join("t1-store.md").exists() && dir.join("t2-auth.md").exists());
    assert!(!dir.join("t3-gate.md").exists(), "the stop is a stop");

    // Nesting refuses; a parse error names its line.
    let (success, _, err) = run_stdin(&root, &["batch"], "batch\n");
    assert!(!success);
    assert!(err.contains("does not nest"), "{err}");
    let (success, _, err) = run_stdin(&root, &["batch"], "plan use 'open\n");
    assert!(!success);
    assert!(err.contains("unterminated"), "{err}");

    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn the_plan_loop_produces_the_links_its_gate_demands() {
    let root = temp_project();

    // A plan pins a hardened spec: refuses before the first save.
    let (_, err) = fails(&root, &["plan", "use", "mvp"]);
    assert!(err.contains("version save"), "{err}");
    ok(&root, &["version", "save", "-m", "first"]);
    let out = ok(&root, &["plan", "use", "mvp"]);
    assert!(out.contains("created plan `mvp` @ v0001"), "{out}");

    // Tasks are cut per node, spec_refs seeded from the pinned model.
    let out = ok(&root, &["plan", "task", "add", "Store", "--desc", "persist rows"]);
    assert!(out.contains("t1 Store"), "{out}");
    assert!(out.contains("Auth.creds wire Store.inn"), "{out}");
    ok(&root, &["plan", "task", "add", "Auth"]);

    // Authoring is a text edit of the record files: outputs scope
    // capture, inputs shape waves, a scenario rides the envelope.
    write_record(
        &root,
        "archi/plans/mvp/scenarios.md",
        "# Scenarios\n\n- a user logs in end to end\n",
    );
    write_record(
        &root,
        "archi/plans/mvp/t1-store.md",
        "---\nnode: Store\nowns: []\n---\n\n# t1 — Store\n\npersist rows\n\n\
         ## Spec\n\n- `Store`\n- `Auth.creds wire Store.inn`\n\n\
         ## Inputs\n\n## Outputs\n\n- code/store.rs\n\n## Stack\n\n## Verifications\n",
    );
    write_record(
        &root,
        "archi/plans/mvp/t2-auth.md",
        "---\nnode: Auth\nowns: []\n---\n\n# t2 — Auth\n\n\
         ## Spec\n\n- `Auth`\n- `Gate.out wire Auth.inn`\n- `Service type_of Auth`\n\n\
         ## Inputs\n\n- from t1 — the store api\n\n\
         ## Outputs\n\n- code/auth.rs\n\n## Stack\n\n## Verifications\n",
    );

    // The start gate: matches are candidates — nothing owned, nothing
    // verified, one task still undescribed: the gate refuses.
    let (_, err) = fails(&root, &["plan", "start"]);
    assert!(err.contains("none owned"), "{err}");
    assert!(err.contains("empty description"), "{err}");

    // The derived view says which; curation is an edit of the same
    // files — own the requirement, author the proof, describe the task.
    // Verify carries the worklist (and exits nonzero while it stands).
    let (verify_out, _) = fails(&root, &["plan", "verify", "--json"]);
    let verify: Value = serde_json::from_str(&verify_out).unwrap();
    assert_eq!(verify["matched"]["t1"][1]["req"], "store-encrypted", "{verify}");
    write_record(
        &root,
        "archi/plans/mvp/t1-store.md",
        "---\nnode: Store\nowns: [store-encrypted]\n---\n\n# t1 — Store\n\npersist rows\n\n\
         ## Spec\n\n- `Store`\n- `Auth.creds wire Store.inn`\n\n\
         ## Inputs\n\n## Outputs\n\n- code/store.rs\n\n## Stack\n\n## Verifications\n\n\
         ### store-encrypted\n\n- test — proves store-encrypted\n",
    );
    write_record(
        &root,
        "archi/plans/mvp/t2-auth.md",
        "---\nnode: Auth\nowns: [service-hardening]\n---\n\n# t2 — Auth\n\nrealize the node\n\n\
         ## Spec\n\n- `Auth`\n- `Gate.out wire Auth.inn`\n- `Service type_of Auth`\n\n\
         ## Inputs\n\n- from t1 — the store api\n\n\
         ## Outputs\n\n- code/auth.rs\n\n## Stack\n\n## Verifications\n\n\
         ### service-hardening\n\n- test — proves service-hardening\n",
    );

    let out = ok(&root, &["plan", "start"]);
    assert!(out.contains("wave 1 in flight: t1"), "{out}");
    let out = ok(&root, &["plan", "current-wave"]);
    assert!(out.contains("t1 Store — persist rows"), "{out}");

    // Close wave 1: the edit under t1's output becomes candidates, and
    // the gate blocks until they are asserted — the step that demands
    // links is the step that produces them.
    fs::write(
        root.join("code/store.rs"),
        "pub struct Store;\nimpl Store {\n    pub fn put(&mut self, n: u8) { let _ = n; }\n}\n",
    )
    .unwrap();
    let (stdout, stderr) = fails(&root, &["plan", "next"]);
    assert!(stderr.contains("coverage of the refs this delta presses is incomplete"), "{stderr}");
    let ids = captured_ids(&stdout);
    assert_eq!(ids.len(), 2, "{stdout}");

    // A manual re-run is idempotent, and `--json` carries the full
    // product: what was pressed, what was suppressed.
    let out = ok(&root, &["link", "capture", "--task", "t1"]);
    assert!(!out.contains("captured "), "{out}");
    let json: Value =
        serde_json::from_str(&ok(&root, &["link", "capture", "--task", "t1", "--json"])).unwrap();
    assert_eq!(json["pressed"]["t1"].as_array().unwrap().len(), 2, "{json}");
    assert!(json["suppressed"].as_array().unwrap().is_empty(), "{json}");

    // Review and assert, then re-run the gate.
    for id in &ids {
        ok(&root, &["link", "confirm", id]);
    }
    let out = ok(&root, &["plan", "next"]);
    assert!(out.contains("wave 1 closed — in flight: t2"), "{out}");

    // Wave 2's delta shares no term with any of t2's refs: nothing is
    // pressed, so nothing gates — the last wave closes into the cleanup
    // wave, the no-signal product suppressed and the untouched surface
    // suggested as a checklist instead of a jam. The cleanup block
    // prints once and latches in state.json; the scenarios wait.
    fs::write(
        root.join("code/auth.rs"),
        "pub fn login(u: &str) -> bool { !u.is_empty() }\n",
    )
    .unwrap();
    let out = ok(&root, &["plan", "next"]);
    assert!(!out.contains("captured "), "{out}");
    assert!(out.contains("suppressed 3 no-signal pair(s)"), "{out}");
    assert!(out.contains("hand-author"), "{out}");
    assert!(out.contains("archi link add \"Auth\" <file#symbol> --kind indirect"), "{out}");
    assert_eq!(out.matches("the cleanup wave").count(), 1, "{out}");
    assert!(!out.contains("a user logs in end to end"), "{out}");
    assert_eq!(state_json(&root, "mvp")["cleanup_displayed"], true);

    // The next call brings the scenarios block exactly as before the
    // cleanup stage existed.
    let out = ok(&root, &["plan", "next"]);
    assert!(out.contains("all waves closed — scenarios:"), "{out}");
    assert!(out.contains("a user logs in end to end"), "{out}");
    assert!(!out.contains("the cleanup wave"), "printed once: {out}");

    // One more next closes the plan — in state.json; the content files
    // never moved, and no plan.json ever appeared.
    let out = ok(&root, &["plan", "next"]);
    assert!(out.contains("DONE"), "{out}");
    assert_eq!(state_json(&root, "mvp")["state"], "completed");
    assert!(!root.join("archi/plans/mvp/plan.json").exists());

    // The checklist is actionable as printed: hand-author the untouched
    // surface, and the links land asserted — covered is covered, however
    // a link is born.
    ok(&root, &["link", "add", "Auth", "code/auth.rs#login", "--kind", "indirect"]);
    ok(&root, &["link", "add", "Gate.out wire Auth.inn", "code/auth.rs#login", "--kind", "indirect"]);
    ok(&root, &["link", "add", "Service type_of Auth", "code/auth.rs", "--kind", "indirect"]);

    // Nothing in the plan's scope is dark now, and the journal holds the
    // captures with their confirms folded in.
    let out = ok(&root, &["link", "audit"]);
    assert!(!out.contains("unlinked spec element"), "{out}");
    let out = ok(&root, &["link", "ls"]);
    assert!(out.contains("captured(t1)"), "{out}");
    assert!(out.contains("authored"), "{out}");

    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn a_legacy_plan_json_reads_forever_and_its_lifecycle_verbs_advance_it() {
    let root = temp_project();
    ok(&root, &["version", "save", "-m", "first"]);

    // The old form, written by hand — no command mints it anymore.
    fs::create_dir_all(root.join("archi/plans/mvp")).unwrap();
    fs::write(
        root.join("archi/plans/mvp/plan.json"),
        r#"{
  "name": "mvp",
  "version": "v0001",
  "created": "2026-01-01T00:00:00Z",
  "state": "draft",
  "closed_waves": 0,
  "problem": "kept as it was",
  "tasks": [
    {
      "id": "t1",
      "node": "Store",
      "description": "persist rows",
      "spec_refs": ["Store", "Auth.creds wire Store.inn"],
      "owns": ["service-hardening", "store-encrypted"],
      "outputs": ["code/store.rs"],
      "verifications": {
        "service-hardening": ["test — hardened"],
        "store-encrypted": ["test — sealed"]
      }
    }
  ]
}
"#,
    )
    .unwrap();

    // The dual read serves it whole: switch, status, show.
    let out = ok(&root, &["plan", "use", "mvp"]);
    assert!(out.contains("switched to plan `mvp` @ v0001"), "{out}");
    assert!(ok(&root, &["plan", "status"]).contains("plan `mvp` @ v0001 (draft)"));
    assert!(ok(&root, &["plan", "show"]).contains("problem: kept as it was"));

    // The form only shrinks: the mint commands refuse.
    let (_, err) = fails(&root, &["plan", "task", "add", "Auth"]);
    assert!(err.contains("legacy plan.json is read-only"), "{err}");
    let (_, err) = fails(&root, &["plan", "task", "rm", "t1"]);
    assert!(err.contains("read-only"), "{err}");

    // Lifecycle still moves the old form: start, next through the
    // cleanup wave to done, reset — written back as plan.json, never as
    // a record folder.
    let out = ok(&root, &["plan", "start"]);
    assert!(out.contains("wave 1 in flight: t1"), "{out}");
    let out = ok(&root, &["plan", "next"]);
    assert!(out.contains("the cleanup wave"), "{out}");
    let out = ok(&root, &["plan", "next"]);
    assert!(out.contains("DONE"), "{out}");
    let text = fs::read_to_string(root.join("archi/plans/mvp/plan.json")).unwrap();
    let plan: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(plan["state"], "completed");
    assert_eq!(plan["problem"], "kept as it was", "content rides along untouched");
    assert!(!root.join("archi/plans/mvp/mvp.md").exists());
    assert!(!root.join("archi/plans/mvp/state.json").exists());
    ok(&root, &["plan", "reset"]);
    assert!(ok(&root, &["plan", "status"]).contains("(draft)"));

    fs::remove_dir_all(&root).unwrap();
}

/// A state.json an old binary wrote — no cleanup latch field — still
/// parses, and the field interplay holds: waves closed with the
/// scenarios already displayed completes without demanding the cleanup
/// stage; waves closed with the scenarios not yet displayed enters it.
/// Reset clears the new latch with the others.
#[test]
fn a_legacy_state_json_never_regresses_into_the_cleanup_stage() {
    let root = temp_project();
    ok(&root, &["version", "save", "-m", "first"]);
    ok(&root, &["plan", "use", "mvp"]);
    ok(&root, &["plan", "task", "add", "Store"]);
    write_record(
        &root,
        "archi/plans/mvp/t1-store.md",
        "---\nnode: Store\nowns: [store-encrypted]\n---\n\n# t1 — Store\n\npersist rows\n\n\
         ## Spec\n\n- `Store`\n- `Auth.creds wire Store.inn`\n\n\
         ## Inputs\n\n## Outputs\n\n- code/store.rs\n\n## Stack\n\n## Verifications\n\n\
         ### store-encrypted\n\n- test — proves store-encrypted\n",
    );
    write_record(
        &root,
        "archi/plans/mvp/scenarios.md",
        "# Scenarios\n\n- a row survives a restart\n",
    );
    let created = state_json(&root, "mvp")["created"].as_str().unwrap().to_string();
    let legacy_state = |latches: &str| {
        format!(
            "{{\n  \"state\": \"started\",\n  \"closed_waves\": 1,\n  \
             \"version\": \"v0001\",\n  \"created\": \"{created}\"{latches}\n}}\n"
        )
    };

    // As the old binary left it mid-dance: waves closed, scenarios
    // displayed — one next completes; the cleanup stage is not demanded.
    write_record(
        &root,
        "archi/plans/mvp/state.json",
        &legacy_state(",\n  \"scenarios_displayed\": true"),
    );
    let out = ok(&root, &["plan", "next"]);
    assert!(out.contains("DONE"), "{out}");
    assert!(!out.contains("the cleanup wave"), "no regression: {out}");
    assert_eq!(state_json(&root, "mvp")["state"], "completed");

    // Waves closed but the scenarios never displayed: the cleanup stage
    // appears, latches, then the scenarios, then done.
    write_record(&root, "archi/plans/mvp/state.json", &legacy_state(""));
    let out = ok(&root, &["plan", "next"]);
    assert_eq!(out.matches("the cleanup wave").count(), 1, "{out}");
    assert_eq!(state_json(&root, "mvp")["cleanup_displayed"], true);
    let out = ok(&root, &["plan", "next"]);
    assert!(out.contains("all waves closed — scenarios:"), "{out}");
    assert!(out.contains("a row survives a restart"), "{out}");
    let out = ok(&root, &["plan", "next"]);
    assert!(out.contains("DONE"), "{out}");

    // Reset clears the cleanup latch like the others: unflipped latches
    // drop out of state.json entirely.
    ok(&root, &["plan", "reset"]);
    let state = state_json(&root, "mvp");
    assert_eq!(state["state"], "draft");
    assert!(state.get("cleanup_displayed").is_none(), "{state}");
    assert!(state.get("scenarios_displayed").is_none(), "{state}");

    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn both_forms_at_once_refuse_and_duplicate_ids_name_both_files() {
    let root = temp_project();
    ok(&root, &["version", "save", "-m", "first"]);
    ok(&root, &["plan", "use", "mvp"]);
    ok(&root, &["plan", "task", "add", "Store"]);

    // plan.json beside the folder is a conflict, not a merge.
    let dir = root.join("archi/plans/mvp");
    fs::write(dir.join("plan.json"), "{}\n").unwrap();
    let (_, err) = fails(&root, &["plan", "status"]);
    assert!(err.contains("carries both plan.json and the record folder"), "{err}");
    assert!(err.contains("keep one"), "{err}");
    fs::remove_file(dir.join("plan.json")).unwrap();
    ok(&root, &["plan", "status"]);

    // Two files claiming one ordinal refuse naming both — the slug part
    // of a task file name is free, the `t<N>-` prefix is the identity.
    fs::copy(dir.join("t1-store.md"), dir.join("t1-zzz.md")).unwrap();
    let (_, err) = fails(&root, &["plan", "status"]);
    assert!(err.contains("duplicate task id `t1`"), "{err}");
    assert!(err.contains("t1-store.md") && err.contains("t1-zzz.md"), "{err}");
    fs::remove_file(dir.join("t1-zzz.md")).unwrap();
    ok(&root, &["plan", "status"]);

    fs::remove_dir_all(&root).unwrap();
}

/// `plan show <name>` is a pure read: it renders any stored plan without
/// consulting or rewriting `.current`, and an unknown name lists what
/// exists.
#[test]
fn a_named_show_reads_any_plan_without_moving_the_marker() {
    let root = temp_project();
    ok(&root, &["version", "save", "-m", "first"]);
    ok(&root, &["plan", "use", "alpha"]);
    ok(&root, &["plan", "use", "beta"]);

    let marker = root.join("archi/plans/.current");
    let before = fs::read_to_string(&marker).unwrap();
    assert_eq!(before.trim(), "beta");

    // The non-active plan renders under its own name and version…
    let out = ok(&root, &["plan", "show", "alpha"]);
    assert!(out.contains("plan `alpha` @ v0001"), "{out}");
    // …and the marker is untouched: `beta` stays current.
    assert_eq!(fs::read_to_string(&marker).unwrap(), before);

    // The nameless form still answers with the active plan.
    let out = ok(&root, &["plan", "show"]);
    assert!(out.contains("plan `beta` @ v0001"), "{out}");

    // JSON rides along the named form unchanged.
    let out = ok(&root, &["plan", "show", "alpha", "--json"]);
    let v: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["plan"]["name"], "alpha");
    assert_eq!(fs::read_to_string(&marker).unwrap(), before);

    // An unknown name lists the plans that do exist.
    let (_, err) = fails(&root, &["plan", "show", "nope"]);
    assert!(err.contains("no plan `nope` — plans: alpha, beta"), "{err}");

    fs::remove_dir_all(&root).unwrap();
}

/// The named show is what an unbound checkout gets: once the worktree's plan
/// lands on `main`, the primary — unbound, no `.current` — reads it by
/// name; the read never mints a marker, and the nameless form still
/// needs one.
#[test]
fn a_named_show_answers_from_an_unbound_checkout() {
    let wt = temp_project();
    ok(&wt, &["version", "save", "-m", "first"]);
    ok(&wt, &["plan", "use", "mvp"]);

    // Land the worktree's work on the primary checkout's branch.
    util::git(&wt, &["add", "-A"]);
    util::git(&wt, &["commit", "-qm", "plan"]);
    let wt_dir = wt.parent().unwrap();
    let name = wt_dir.file_name().unwrap().to_str().unwrap();
    let primary = wt_dir
        .parent()
        .unwrap()
        .join(name.strip_suffix("-worktrees").unwrap());
    util::git(&primary, &["merge", "-q", "archi/wt"]);

    // The marker is machine-local by construction — mint's repo-local
    // exclude keeps it out of any commit — so the primary carries none.
    let marker = primary.join("archi/plans/.current");
    assert!(!marker.exists(), "the marker never travels");

    let out = ok(&primary, &["plan", "show", "mvp"]);
    assert!(out.contains("plan `mvp` @ v0001"), "{out}");
    assert!(!marker.exists(), "a named show never writes the marker");

    // The nameless form still needs the marker — the named form is the
    // unbound checkout's read.
    let (_, err) = fails(&primary, &["plan", "show"]);
    assert!(err.contains("no active plan"), "{err}");

    fs::remove_dir_all(&wt).unwrap();
    fs::remove_dir_all(&primary).unwrap();
}
