//! End to end through the real binary: a plan projects a hardened spec
//! into tasks, waves gate on captured-then-asserted code-links, and the
//! scenario latch closes the cycle (`archi/requirements/planning/`,
//! `archi/requirements/self-hosting/capture-at-the-join.md`).

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
    dir
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

fn plan_json(root: &Path) -> Value {
    let text = fs::read_to_string(root.join("archi/plans/mvp/plan.json")).unwrap();
    serde_json::from_str(&text).unwrap()
}

fn store_plan_json(root: &Path, plan: &Value) {
    fs::write(
        root.join("archi/plans/mvp/plan.json"),
        serde_json::to_string_pretty(plan).unwrap(),
    )
    .unwrap();
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
fn the_authoring_verbs_cover_the_old_surface() {
    let root = temp_project();
    ok(&root, &["version", "save", "-m", "first"]);
    ok(&root, &["plan", "use", "mvp"]);

    // Envelope through verbs.
    ok(&root, &["plan", "problem", "a tiny hardened store"]);
    ok(&root, &["plan", "tech", "add", "Rust", "--provenance", "user choice"]);
    let (_, err) = fails(&root, &["plan", "tech", "add", "Rust"]);
    assert!(err.contains("already in the stack"), "{err}");
    ok(&root, &["plan", "architecture-summary", "add", "Store", "keeps the rows"]);
    ok(&root, &["plan", "architecture-summary", "add", "Auth", "guards the door"]);
    ok(&root, &["plan", "architecture-summary", "add", "Gate", "fronts the world"]);
    ok(&root, &["plan", "stack-mapping", "add", "Store", "Rust"]);
    ok(&root, &["plan", "stack-mapping", "add", "Auth", "Rust"]);
    ok(&root, &["plan", "stack-mapping", "add", "Gate", "Rust"]);
    let (_, err) = fails(&root, &["plan", "stack-mapping", "add", "Store", "Rust"]);
    assert!(err.contains("already realizes"), "{err}");
    assert!(ok(&root, &["plan", "show"]).contains("problem: a tiny hardened store"));

    // Scenarios: add, list, remove — 1-based.
    ok(&root, &["plan", "scenarios", "add", "a user stores a row"]);
    ok(&root, &["plan", "scenarios", "add", "a bogus flow"]);
    assert!(ok(&root, &["plan", "scenarios", "list"]).contains("2. a bogus flow"));
    ok(&root, &["plan", "scenarios", "remove", "2"]);
    let (_, err) = fails(&root, &["plan", "scenarios", "remove", "5"]);
    assert!(err.contains("no scenario 5"), "{err}");

    // Tasks and their authored fields.
    ok(&root, &["plan", "task", "add", "Store"]);
    ok(&root, &["plan", "task", "add", "Auth"]);
    ok(&root, &["plan", "task", "desc", "t1", "persist rows"]);
    ok(&root, &["plan", "task", "desc", "t2", "guard the door"]);
    ok(&root, &["plan", "task", "stack-detail", "add", "t1", "sqlite via rusqlite"]);
    ok(&root, &["plan", "task", "output", "add", "t1", "code/store.rs"]);
    ok(&root, &["plan", "task", "output", "add", "t2", "code/auth.rs"]);
    ok(&root, &["plan", "task", "input", "add", "t2", "the store api", "--from", "t1"]);
    let (_, err) = fails(&root, &["plan", "task", "input", "add", "t2", "ghost", "--from", "t9"]);
    assert!(err.contains("no producer `t9`"), "{err}");
    let (_, err) = fails(&root, &["plan", "task", "spec-ref", "add", "t2", "Phantom"]);
    assert!(err.contains("E_MODEL_REF"), "{err}");
    // widening: the outgoing edge the seed leaves out grows the
    // candidate set of the task that claimed it
    ok(&root, &["plan", "task", "spec-ref", "add", "t2", "Auth.creds wire Store.inn"]);
    let out = ok(&root, &["plan", "task", "req", "suggest", "t2"]);
    assert!(out.contains("store-encrypted (candidate)"), "{out}");

    // Curation: suggest → own (slug or slot) → verification; misuse refuses.
    let out = ok(&root, &["plan", "task", "req", "suggest", "t1"]);
    assert!(out.contains("store-encrypted (candidate)"), "{out}");
    let (_, err) = fails(
        &root,
        &["plan", "task", "verification", "add", "t1", "store-encrypted", "test — sealed"],
    );
    assert!(err.contains("own it first"), "{err}");
    ok(&root, &["plan", "task", "req", "add", "t1", "store-encrypted"]);
    ok(&root, &["plan", "task", "req", "add", "t1", "r1"]); // slot addressing
    let (_, err) = fails(&root, &["plan", "task", "req", "add", "t1", "ghost-req"]);
    assert!(err.contains("not among"), "{err}");
    ok(&root, &[
        "plan", "task", "verification", "add", "t1", "store-encrypted",
        "test — rows encrypted at rest",
    ]);
    ok(&root, &["plan", "task", "verification", "add", "t1", "r1", "test — hardened"]);
    ok(&root, &["plan", "task", "req", "add", "t2", "service-hardening"]);
    ok(&root, &[
        "plan", "task", "verification", "add", "t2", "service-hardening",
        "test — login hardened",
    ]);
    ok(&root, &["plan", "task", "verification", "add", "t2", "service-hardening", "test — drop me"]);
    let (_, err) = fails(&root, &["plan", "task", "verification", "remove", "t2", "service-hardening", "9"]);
    assert!(err.contains("no verification 9"), "{err}");
    ok(&root, &["plan", "task", "verification", "remove", "t2", "service-hardening", "2"]);
    let out = ok(&root, &["plan", "task", "req-list", "t2"]);
    assert!(out.contains("service-hardening — 1 verification"), "{out}");
    // remove without an index clears the key whole — and re-authoring works
    ok(&root, &["plan", "task", "verification", "remove", "t2", "service-hardening"]);
    let out = ok(&root, &["plan", "task", "req-list", "t2"]);
    assert!(out.contains("service-hardening — 0 verifications"), "{out}");
    ok(&root, &[
        "plan", "task", "verification", "add", "t2", "service-hardening",
        "test — login hardened",
    ]);

    // Disowning takes the verifications with it.
    ok(&root, &["plan", "task", "req", "remove", "t1", "service-hardening"]);
    let out = ok(&root, &["plan", "task", "req-list", "t1"]);
    assert!(!out.contains("service-hardening"), "{out}");
    assert!(out.contains("store-encrypted"), "{out}");

    // The standalone brief.
    let brief = ok(&root, &["plan", "task", "show", "t1"]);
    assert!(brief.contains("t1 Store — persist rows"), "{brief}");
    assert!(brief.contains("sqlite via rusqlite"), "{brief}");
    assert!(brief.contains("output: code/store.rs"), "{brief}");
    assert!(brief.contains("store-encrypted (owned"), "{brief}");
    assert!(brief.contains("verify: test — rows encrypted at rest"), "{brief}");

    // The read surfaces, then the verb-authored plan verifies and starts.
    assert!(ok(&root, &["plan", "list"]).contains("mvp @ v0001 (draft)"));
    assert!(ok(&root, &["plan", "status"]).contains("plan `mvp` @ v0001 (draft), 0 waves closed"));
    ok(&root, &["plan", "verify"]);
    ok(&root, &["plan", "start"]);
    assert!(ok(&root, &["plan", "status"]).contains("(started)"));

    // Authoring is a draft-stage act.
    let (_, err) = fails(&root, &["plan", "problem", "too late"]);
    assert!(err.contains("authoring happens in draft"), "{err}");

    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn batch_runs_every_verb_from_stdin_and_stops_at_the_first_failure() {
    let root = temp_project();
    ok(&root, &["version", "save", "-m", "first"]);

    // One invocation authors the whole plan: comments and blank lines are
    // skipped, CRLF tolerated, `\n` in double quotes carries a multiline
    // value, `--project` is forwarded to every line.
    let script = "# the whole plan in one call\r\n\
                  plan use mvp\n\
                  plan problem \"a tiny hardened store\"\n\
                  plan tech add Rust --provenance \"user choice\"\n\
                  \n\
                  plan architecture-summary add Store \"keeps the rows\"\n\
                  plan architecture-summary add Auth \"guards the door\"\n\
                  plan architecture-summary add Gate \"fronts the world\"\n\
                  plan stack-mapping add Store Rust\n\
                  plan stack-mapping add Auth Rust\n\
                  plan stack-mapping add Gate Rust\n\
                  plan scenarios add \"a user stores a row\"\n\
                  plan task add Store\n\
                  plan task add Auth\n\
                  plan task desc t1 \"persist rows\\nacross restarts\"\n\
                  plan task desc t2 \"guard the door\"\n\
                  plan task output add t1 code/store.rs\n\
                  plan task output add t2 code/auth.rs\n\
                  plan task input add t2 \"the store api\" --from t1\n\
                  plan task req add t1 store-encrypted\n\
                  plan task req add t2 service-hardening\n\
                  plan task verification add t1 store-encrypted \"test — rows encrypted at rest\"\n\
                  plan task verification add t2 service-hardening \"test — login hardened\"\n\
                  plan task req remove t1 service-hardening\n";
    let (success, out, err) = run_stdin(&root, &["batch"], script);
    // t1 never owned service-hardening — the batch stops exactly there.
    assert!(!success, "{out}");
    assert!(err.contains("batch stopped at line 24"), "{err}");
    assert!(out.contains("[22]"), "the lines before it ran: {out}");

    // Everything before the stop is applied; nothing after existed.
    let brief = ok(&root, &["plan", "task", "show", "t1"]);
    assert!(brief.contains("across restarts"), "the escaped newline landed: {brief}");
    ok(&root, &["plan", "verify"]);
    ok(&root, &["plan", "start"]);

    // Nesting refuses; a parse error names its line.
    let (success, _, err) = run_stdin(&root, &["batch"], "batch\n");
    assert!(!success);
    assert!(err.contains("does not nest"), "{err}");
    let (success, _, err) = run_stdin(&root, &["batch"], "plan problem 'open\n");
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

    // Authoring is a text edit of plan.json: outputs scope capture,
    // inputs shape waves, a scenario rides the envelope.
    let mut plan = plan_json(&root);
    plan["scenarios"] = serde_json::json!(["a user logs in end to end"]);
    plan["tasks"][0]["outputs"] = serde_json::json!(["code/store.rs"]);
    plan["tasks"][1]["outputs"] = serde_json::json!(["code/auth.rs"]);
    plan["tasks"][1]["inputs"] = serde_json::json!({"t1": "the store api"});
    store_plan_json(&root, &plan);

    // The start gate: matches are candidates — nothing owned, nothing
    // verified, one task still undescribed: the gate refuses.
    let (_, err) = fails(&root, &["plan", "start"]);
    assert!(err.contains("none owned"), "{err}");
    assert!(err.contains("empty description"), "{err}");

    // The derived view says which — curate ownership and author the
    // proofs through the same text surface. Verify carries the worklist
    // (and exits nonzero while it stands).
    let (verify_out, _) = fails(&root, &["plan", "verify", "--json"]);
    let verify: Value = serde_json::from_str(&verify_out).unwrap();
    let mut plan = plan_json(&root);
    for task in plan["tasks"].as_array_mut().unwrap() {
        let id = task["id"].as_str().unwrap().to_string();
        if task["description"].as_str().unwrap_or_default().is_empty() {
            task["description"] = serde_json::json!("realize the node");
        }
        let mut owns = Vec::new();
        let mut proofs = serde_json::Map::new();
        for m in verify["matched"][&id].as_array().unwrap() {
            let req = m["req"].as_str().unwrap();
            owns.push(req.to_string());
            proofs.insert(req.into(), serde_json::json!([format!("test — proves {req}")]));
        }
        task["owns"] = serde_json::json!(owns);
        task["verifications"] = Value::Object(proofs);
    }
    store_plan_json(&root, &plan);

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
    // pressed, so nothing gates — the wave closes straight to the
    // scenarios, the no-signal product suppressed and the untouched
    // surface suggested as a checklist instead of a jam.
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
    assert!(out.contains("a user logs in end to end"), "{out}");

    // One more next closes the plan.
    let out = ok(&root, &["plan", "next"]);
    assert!(out.contains("DONE"), "{out}");
    assert_eq!(plan_json(&root)["state"], "completed");

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
