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
    // The tests a declaration names. They stand before any wave opens, so
    // they sit in every wave-open index and are never a change of their own.
    fs::write(dir.join("code/tests.rs"), TESTS_RS).unwrap();
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

/// The curated t1 Store record — requirement owned, proof authored —
/// shared by the tests that drive the lifecycle past the start gate.
const T1_STORE_CURATED: &str =
    "---\nnode: Store\nowns: [store-encrypted]\n---\n\n# t1 — Store\n\npersist rows\n\n\
     ## Spec\n\n- `Store`\n- `Auth.creds wire Store.inn`\n\n\
     ## Inputs\n\n## Outputs\n\n- code/store.rs\n\n## Stack\n\n## Verifications\n\n\
     ### store-encrypted\n\n- test — proves store-encrypted\n";

/// One world fact under `archi/world/facts/`, in the shape `world add` mints
/// and a person fills: the three lists, the conditioning paragraph, the
/// workaround and a `Scenarios` block (`archi/requirements/world-facts/`).
fn put_fact(root: &Path, slug: &str, title: &str, covers: &str, scenarios: &[&str]) {
    let with_steps: Vec<(&str, &[&str])> = scenarios.iter().map(|s| (*s, STEPS)).collect();
    put_fact_with_steps(root, slug, title, covers, &with_steps);
}

/// The three steps every scenario carries unless the test spells its own out.
const STEPS: &[&str] = &[
    "Given the carriage leaves the platform",
    "When the rider opens the door",
    "Then the door holds",
];

/// The same fixture with the steps spelled out, so a test can reword one
/// step and watch the witness part — `put_fact`'s block with its Gherkin
/// under the test's control. A scenario is a `### ` heading and its steps:
/// the fact's own title is the feature, so the block names none
/// (`archi/requirements/world-facts/the-grammar-is-a-named-subset.md`).
fn put_fact_with_steps(
    root: &Path,
    slug: &str,
    title: &str,
    covers: &str,
    scenarios: &[(&str, &[&str])],
) {
    let mut block = String::new();
    for (name, steps) in scenarios {
        block.push_str(&format!("### {name}\n\n"));
        for step in *steps {
            block.push_str(&format!("{step}\n"));
        }
        block.push('\n');
    }
    util::Fact {
        covers,
        sources: "https://example.org/thread/42",
        uses: "",
        condition: "The carriage drops the network for minutes at a time.",
        workaround: "Riders load the page at the platform and redo what the drop takes.",
        scenarios: &block,
    }
    .write(root, slug, title);
}

/// This family's declaration of what no fact of it reaches: the shared one
/// ([`util::declare_internal`]). A test here writes the one fact that covers
/// the node its task sits on, and the gate on `version save` asks about the
/// whole model; the nodes left over go through this.
fn declare_internal(root: &Path, nodes: &[&str]) {
    util::declare_internal(root, nodes);
}

/// This family's shim: the shared one ([`util::shim`]) under this family's
/// own scratch name.
fn shim(root: &Path) -> PathBuf {
    util::shim(root, "archi-plan-e2e")
}

/// Run one line through `sh`, exactly as it was printed ([`util::shell`]).
fn shell(bin: &Path, line: &str) -> (Option<i32>, String, String) {
    util::shell(bin, line)
}

/// The one `archi link add` line a transcript printed, trimmed of the
/// bullet the caller marked it with.
fn printed_link_add(out: &str) -> String {
    let lines: Vec<&str> = out
        .lines()
        .map(str::trim)
        .filter(|l| l.starts_with("archi link add"))
        .collect();
    assert_eq!(lines.len(), 1, "one command, for the one unanchored scenario: {out}");
    lines[0].to_string()
}

/// Curate a minted task file the way a person does: own one requirement,
/// author its proof, name the file the task writes — and leave the
/// machine-written frontmatter exactly as the mint left it, carried facts
/// and all.
fn curate(root: &Path, rel: &str, owns: &str, output: &str) {
    let path = root.join(rel);
    let text = fs::read_to_string(&path)
        .unwrap()
        .replace("owns: []", &format!("owns: [{owns}]"))
        .replace("## Outputs\n", &format!("## Outputs\n\n- {output}\n"));
    fs::write(&path, format!("{text}\n### {owns}\n\n- test — proves {owns}\n")).unwrap();
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

    // `use` mints the folder: the charter skeleton and the lifecycle in
    // state.json — no plan.json is ever born again, and no `scenarios.md`:
    // the plan authors no stories, it collects them from the world.
    let out = ok(&root, &["plan", "use", "mvp"]);
    assert!(out.contains("created plan `mvp` @ v0001"), "{out}");
    let dir = root.join("archi/plans/mvp");
    assert_eq!(
        fs::read_to_string(dir.join("mvp.md")).unwrap(),
        "# mvp\n\n## Stack\n\n## Architecture\n"
    );
    assert!(!dir.join("scenarios.md").exists());
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
    // The plan's own block is retired: no fact covers its nodes, so the
    // collected set is empty and the listing says so.
    assert!(ok(&root, &["plan", "scenarios", "list"]).contains("no scenarios"));
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
    put_fact(
        &root,
        "riders-lose-the-signal",
        "Riders lose the signal",
        "Auth",
        &["a user logs in end to end"],
    );

    // A plan pins a hardened spec: refuses before the first save.
    let (_, err) = fails(&root, &["plan", "use", "mvp"]);
    assert!(err.contains("version save"), "{err}");
    // The fact reaches `Auth` and `Store` behind it; `Gate` is the node it never
    // touches, and the save's gate takes the declaration as its second exit.
    declare_internal(&root, &["Gate"]);
    ok(&root, &["version", "save", "-m", "first"]);
    let out = ok(&root, &["plan", "use", "mvp"]);
    assert!(out.contains("created plan `mvp` @ v0001"), "{out}");

    // Tasks are cut per node, spec_refs seeded from the pinned model.
    let out = ok(&root, &["plan", "task", "add", "Store", "--desc", "persist rows"]);
    assert!(out.contains("t1 Store"), "{out}");
    assert!(out.contains("Auth.creds wire Store.inn"), "{out}");
    ok(&root, &["plan", "task", "add", "Auth"]);

    // Authoring is a text edit of the record files: outputs scope
    // capture, inputs shape waves. The stories are the world's.
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
    write_record(&root, "archi/plans/mvp/t1-store.md", T1_STORE_CURATED);
    write_record(
        &root,
        "archi/plans/mvp/t2-auth.md",
        "---\nnode: Auth\nowns: [service-hardening]\n---\n\n# t2 — Auth\n\nrealize the node\n\n\
         ## Spec\n\n- `Auth`\n- `Gate.out wire Auth.inn`\n- `Service type_of Auth`\n\n\
         ## Inputs\n\n- from t1 — the store api\n\n\
         ## Outputs\n\n- code/auth.rs\n\n## Stack\n\n## Verifications\n\n\
         ### service-hardening\n\n- test — proves service-hardening\n",
    );

    // The tests the declarations name, in the tree before the wave opens so
    // they are never a change of their own.
    fs::write(
        root.join("code/store_test.rs"),
        "pub fn a_row_is_persisted() {\n    assert!(true);\n}\n",
    )
    .unwrap();
    fs::write(
        root.join("code/auth_test.rs"),
        "pub fn a_login_without_a_name_is_refused() {\n    assert!(true);\n}\n",
    )
    .unwrap();
    let out = ok(&root, &["plan", "start"]);
    assert!(out.contains("wave 1 in flight: t1"), "{out}");
    let out = ok(&root, &["plan", "current-wave"]);
    assert!(out.contains("t1 Store — persist rows"), "{out}");

    // Close wave 1: the edit under t1's output moves a symbol t1 claims, and
    // the wave refuses while nothing declares it. Nothing was captured — the
    // diff proves a symbol moved, it never proves what that symbol answers.
    fs::write(
        root.join("code/store.rs"),
        "pub struct Store;\nimpl Store {\n    pub fn put(&mut self, n: u8) { let _ = n; }\n}\n",
    )
    .unwrap();
    let (stdout, stderr) = fails(&root, &["plan", "next"]);
    assert!(stderr.contains("t1 — write"), "{stderr}");
    assert!(captured_ids(&stdout).is_empty(), "{stdout}");
    assert!(stdout.contains("w01.t1.declares.toml"), "{stdout}");

    // A manual re-run is idempotent, and `--json` carries the full
    // product: what was pressed, what was suppressed.
    let out = ok(&root, &["link", "capture", "--task", "t1"]);
    assert!(!out.contains("captured "), "{out}");
    let json: Value =
        serde_json::from_str(&ok(&root, &["link", "capture", "--task", "t1", "--json"])).unwrap();
    assert_eq!(json["pressed"]["t1"].as_array().unwrap().len(), 2, "{json}");
    assert!(json["suppressed"].as_array().unwrap().is_empty(), "{json}");

    // The writer declares what its symbol answers and the test that proves
    // it: the pair lands asserted, with no review step between the claim and
    // the record. The other pressed ref is an edge, and an edge is never a
    // declaration's to name, so it is hand-authored — which is what the
    // refusal printed.
    declares(
        &root,
        1,
        "t1",
        &[[
            "code/store.rs#Store::put",
            "Store",
            "code/store_test.rs#a_row_is_persisted",
        ]],
    );
    let (stdout, stderr) = fails(&root, &["plan", "next"]);
    assert_eq!(captured_ids(&stdout).len(), 1, "{stdout}");
    assert!(stderr.contains("coverage of the refs this delta presses is incomplete"), "{stderr}");
    ok(&root, &[
        "link", "add", "Auth.creds wire Store.inn", "code/store.rs#Store::put",
        "--kind", "indirect",
    ]);
    let out = ok(&root, &["plan", "next"]);
    assert!(captured_ids(&out).is_empty(), "the pair is held, not minted twice: {out}");
    assert!(out.contains("wave 1 closed — in flight: t2"), "{out}");
    let declared = ok(&root, &["link", "ls", "--spec", "Store"]);
    assert!(declared.contains("asserted"), "{declared}");
    assert!(declared.contains("declared"), "{declared}");
    assert!(
        declared.contains("proved by code/store_test.rs#a_row_is_persisted"),
        "{declared}"
    );

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
    declares(
        &root,
        2,
        "t2",
        &[[
            "code/auth.rs#login",
            "req:service-hardening",
            "code/auth_test.rs#a_login_without_a_name_is_refused",
        ]],
    );
    let out = ok(&root, &["plan", "next"]);
    assert_eq!(captured_ids(&out).len(), 1, "the declaration mints its pair: {out}");
    assert!(out.contains("suppressed 3 no-signal pair(s)"), "{out}");
    assert!(out.contains("hand-author"), "{out}");
    assert!(out.contains("archi link add \"Auth\" <file#symbol> --kind indirect"), "{out}");
    assert_eq!(out.matches("the cleanup wave").count(), 1, "{out}");
    assert!(!out.contains("a user logs in end to end"), "{out}");
    assert_eq!(state_json(&root, "mvp")["cleanup_displayed"], true);

    // The next call brings the block, collected from the world: the fact
    // covering t2's node dictates the story this plan closes on.
    let out = ok(&root, &["plan", "next"]);
    assert!(out.contains("all waves closed — scenarios:"), "{out}");
    assert!(
        out.contains("riders-lose-the-signal#a user logs in end to end"),
        "{out}"
    );
    assert!(!out.contains("the cleanup wave"), "printed once: {out}");

    // The latch proves the block is attached to code: anchor the scenario,
    // then one more next closes the plan — in state.json; the content files
    // never moved, and no plan.json ever appeared.
    let (_, err) = fails(&root, &["plan", "next"]);
    assert!(err.contains("no link"), "{err}");
    ok(
        &root,
        &[
            "link",
            "add",
            "riders-lose-the-signal#a user logs in end to end",
            "code/auth.rs",
            "--kind",
            "indirect",
        ],
    );
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
    declares(&root, 1, "t1", &[STORE_ENTRY]);
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
    write_record(&root, "archi/plans/mvp/t1-store.md", T1_STORE_CURATED);
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
    // appears, latches, then the close. The plan is one from before the
    // world — no fact covers its node, so it closes with no block, and the
    // `scenarios.md` it was written with is read by nobody.
    write_record(&root, "archi/plans/mvp/state.json", &legacy_state(""));
    let out = ok(&root, &["plan", "next"]);
    assert_eq!(out.matches("the cleanup wave").count(), 1, "{out}");
    assert_eq!(state_json(&root, "mvp")["cleanup_displayed"], true);
    let out = ok(&root, &["plan", "next"]);
    assert!(out.contains("DONE"), "{out}");
    assert!(!out.contains("a row survives a restart"), "{out}");

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

/// A task carries the world facts covering its node — the slug and a
/// fingerprint, never the story — and every read re-resolves them:
/// `plan task show` lists them beside the requirements, `plan verify` names
/// what moved, `plan repin` adopts the new picture
/// (`archi/requirements/world-facts/a-task-carries-the-facts-that-cover-its-node.md`).
#[test]
fn a_task_carries_the_facts_that_cover_its_node() {
    let root = temp_project();
    put_fact(
        &root,
        "riders-lose-the-signal",
        "Riders lose the signal",
        "Store",
        &["the app opens with no network"],
    );
    // The fact covers `Store` alone — by design, so the second task carries no
    // fact — so the two nodes above it are declared for the save's gate.
    declare_internal(&root, &["Gate", "Auth"]);
    ok(&root, &["version", "save", "-m", "first"]);
    ok(&root, &["plan", "use", "mvp"]);
    ok(&root, &["plan", "task", "add", "Store", "--desc", "persist rows"]);
    ok(&root, &["plan", "task", "add", "Auth", "--desc", "guard the door"]);

    // The covered node's task carries the slug and the fingerprint; the
    // uncovered node's task carries none, and neither holds a story.
    let t1 = fs::read_to_string(root.join("archi/plans/mvp/t1-store.md")).unwrap();
    assert!(t1.contains("facts: [riders-lose-the-signal@"), "{t1}");
    assert!(
        !t1.contains("the app opens with no network"),
        "the record holds slugs, never scenario text: {t1}"
    );
    let t2 = fs::read_to_string(root.join("archi/plans/mvp/t2-auth.md")).unwrap();
    assert!(!t2.contains("facts:"), "{t2}");

    curate(&root, "archi/plans/mvp/t1-store.md", "store-encrypted", "code/store.rs");
    curate(&root, "archi/plans/mvp/t2-auth.md", "service-hardening", "code/auth.rs");

    // The brief a sub-agent reads names the fact beside the requirements.
    let brief = ok(&root, &["plan", "task", "show", "t1"]);
    assert!(
        brief.contains("fact: riders-lose-the-signal — the app opens with no network"),
        "{brief}"
    );
    assert!(!ok(&root, &["plan", "task", "show", "t2"]).contains("fact:"));

    // A fact retired since the pin is drift, reported on demand — never an
    // error, because the plan may finish against the picture it planned for.
    fs::remove_file(root.join("archi/world/facts/riders-lose-the-signal.md")).unwrap();
    let out = ok(&root, &["plan", "verify"]);
    assert!(
        out.contains("drift: world fact `riders-lose-the-signal` retired"),
        "{out}"
    );
    assert!(out.contains("t1"), "{out}");

    // A fact that began covering the node after the pin is drift too.
    put_fact(
        &root,
        "tunnels-run-long",
        "Tunnels run long",
        "Store",
        &["the tunnel ends"],
    );
    let out = ok(&root, &["plan", "verify"]);
    assert!(out.contains("`tunnels-run-long` now covers"), "{out}");

    // `plan repin` re-resolves the covering facts against the new version:
    // the record carries what covers the node now, and the drift is gone.
    fs::write(root.join("archi/src/extra.arch"), "def node Ledger\n").unwrap();
    // The new node arrives unreached like the other two.
    declare_internal(&root, &["Gate", "Auth", "Ledger"]);
    ok(&root, &["version", "save", "-m", "second"]);
    ok(&root, &["plan", "repin"]);
    let t1 = fs::read_to_string(root.join("archi/plans/mvp/t1-store.md")).unwrap();
    assert!(t1.contains("facts: [tunnels-run-long@"), "{t1}");
    assert!(!t1.contains("riders-lose-the-signal"), "{t1}");
    let out = ok(&root, &["plan", "verify"]);
    assert!(!out.contains("drift:"), "{out}");

    fs::remove_dir_all(&root).unwrap();
}

/// The close collects the block from the world as it stands: one entry per
/// fact covering a node the plan holds a task for, the mark of what lies
/// outside beside it, and the drift above it
/// (`archi/requirements/world-facts/the-plan-closes-on-the-world-s-scenarios.md`,
/// `archi/requirements/world-facts/the-block-marks-what-lies-outside-the-plan.md`,
/// `archi/requirements/world-facts/the-close-re-reads-the-world-and-says-what-moved.md`).
#[test]
fn the_close_collects_the_world_and_marks_what_lies_outside() {
    let root = temp_project();
    fs::write(root.join("archi/src/extra.arch"), "def node Ledger\n").unwrap();
    // One fact over both of the plan's nodes, and one reaching past the
    // plan into nodes it never builds.
    put_fact(
        &root,
        "riders-lose-the-signal",
        "Riders lose the signal",
        "Store, Auth",
        &["the app opens with no network"],
    );
    put_fact(
        &root,
        "tunnels-run-long",
        "Tunnels run long",
        "Store, Gate, Ledger",
        &["the tunnel ends"],
    );
    ok(&root, &["version", "save", "-m", "first"]);
    ok(&root, &["plan", "use", "mvp"]);
    ok(&root, &["plan", "task", "add", "Store", "--desc", "persist rows"]);
    ok(&root, &["plan", "task", "add", "Auth", "--desc", "guard the door"]);
    curate(&root, "archi/plans/mvp/t1-store.md", "store-encrypted", "code/store.rs");
    curate(&root, "archi/plans/mvp/t2-auth.md", "service-hardening", "code/auth.rs");

    ok(&root, &["plan", "start"]);
    declares(&root, 1, "t1", &[STORE_ENTRY]);
    declares(&root, 1, "t2", &[AUTH_ENTRY]);
    let out = ok(&root, &["plan", "next"]);
    assert!(out.contains("the cleanup wave"), "{out}");

    // The world moves under the plan between the pin and the close: the
    // fingerprint the tasks carried no longer matches.
    put_fact(
        &root,
        "riders-lose-the-signal",
        "Riders lose the signal",
        "Store, Auth",
        &["the app opens with no network at all"],
    );

    let out = ok(&root, &["plan", "next"]);
    assert!(out.contains("all waves closed — scenarios:"), "{out}");
    assert_eq!(
        out.matches("riders-lose-the-signal#the app opens with no network at all — unanchored")
            .count(),
        1,
        "a fact covering two of the plan's nodes prints once: {out}"
    );
    assert!(out.contains("tunnels-run-long#the tunnel ends — unanchored"), "{out}");
    // The mark names the node paths the plan never built, and a fact whose
    // covered nodes the plan all holds prints clean.
    assert!(
        out.contains("tunnels-run-long also covers Gate, Ledger — outside this plan"),
        "{out}"
    );
    assert!(!out.contains("riders-lose-the-signal also covers"), "{out}");
    // The drift rides above the block, naming what moved since authoring.
    assert!(
        out.contains("drift: world fact `riders-lose-the-signal` moved"),
        "{out}"
    );

    // The read surface serves the same set the close collected.
    let show = ok(&root, &["plan", "show"]);
    assert!(
        show.contains("scenario: riders-lose-the-signal#the app opens with no network at all"),
        "{show}"
    );
    assert!(show.contains("scenario: tunnels-run-long#the tunnel ends"), "{show}");
    // And so does the listing: the same block, numbered, in slug order —
    // each scenario whole, with its state and the line that anchors it.
    let listed = ok(&root, &["plan", "scenarios", "list"]);
    assert!(!listed.contains("no scenarios"), "{listed}");
    assert_eq!(
        listed,
        "1. riders-lose-the-signal#the app opens with no network at all — unanchored\n    \
         Given the carriage leaves the platform\n    \
         When the rider opens the door\n    \
         Then the door holds\n    \
         archi link add 'riders-lose-the-signal#the app opens with no network at all' \
         <file#symbol> --kind indirect\n\
         2. tunnels-run-long#the tunnel ends — unanchored\n    \
         Given the carriage leaves the platform\n    \
         When the rider opens the door\n    \
         Then the door holds\n    \
         archi link add 'tunnels-run-long#the tunnel ends' <file#symbol> --kind indirect\n\
         3. tunnels-run-long also covers Gate, Ledger — outside this plan\n"
    );

    fs::remove_dir_all(&root).unwrap();
}

/// The final latch proves the block is attached to code: a scenario with no
/// link refuses it by name, an anchored block latches, and `plan reset`
/// clears the latch after a refusal
/// (`archi/requirements/world-facts/the-close-gates-on-anchored-scenarios.md`).
#[test]
fn the_close_gates_on_anchored_scenarios() {
    let root = temp_project();
    put_fact(
        &root,
        "riders-lose-the-signal",
        "Riders lose the signal",
        "Store",
        &["the app opens with no network"],
    );
    // The fact covers `Store`; the nodes above it are the save's gate, not this
    // test's subject.
    declare_internal(&root, &["Gate", "Auth"]);
    ok(&root, &["version", "save", "-m", "first"]);
    ok(&root, &["plan", "use", "mvp"]);
    ok(&root, &["plan", "task", "add", "Store", "--desc", "persist rows"]);
    curate(&root, "archi/plans/mvp/t1-store.md", "store-encrypted", "code/store.rs");
    ok(&root, &["plan", "start"]);
    declares(&root, 1, "t1", &[STORE_ENTRY]);
    ok(&root, &["plan", "next"]);
    let out = ok(&root, &["plan", "next"]);
    assert!(out.contains("riders-lose-the-signal#the app opens with no network"), "{out}");

    // Unanchored: the latch refuses, names the scenario, and the plan
    // stays open — the refusal exits like the plan's other gates.
    let refused = Command::new(env!("CARGO_BIN_EXE_archi"))
        .args(["plan", "next", "--project", root.to_str().unwrap()])
        .output()
        .expect("archi runs");
    assert_eq!(refused.status.code(), Some(1));
    let (_, err) = fails(&root, &["plan", "next"]);
    assert!(err.contains("no link"), "{err}");
    assert!(
        err.contains("riders-lose-the-signal#the app opens with no network"),
        "{err}"
    );
    assert_eq!(state_json(&root, "mvp")["state"], "started");
    assert!(state_json(&root, "mvp").get("scenarios_closed").is_none());

    // Reset clears the latch after the refusal, as it always did.
    ok(&root, &["plan", "reset"]);
    let state = state_json(&root, "mvp");
    assert_eq!(state["state"], "draft");
    assert!(state.get("scenarios_displayed").is_none(), "{state}");

    // Anchor the scenario and run the ceremony again: the block latches.
    ok(
        &root,
        &[
            "link",
            "add",
            "riders-lose-the-signal#the app opens with no network",
            "code/store.rs",
            "--kind",
            "indirect",
        ],
    );
    ok(&root, &["plan", "start"]);
    // The reset took the waves folder with it, declarations and all: the
    // second run of the ceremony writes the formality again.
    declares(&root, 1, "t1", &[STORE_ENTRY]);
    ok(&root, &["plan", "next"]);
    ok(&root, &["plan", "next"]);
    let out = ok(&root, &["plan", "next"]);
    assert!(out.contains("DONE"), "{out}");
    assert_eq!(state_json(&root, "mvp")["state"], "completed");

    fs::remove_dir_all(&root).unwrap();
}

/// On a tree that holds a world, a plan minted after it cannot close on
/// nothing: the empty block refuses the final latch and names the reason, and
/// one fact covering a node the plan holds a task for closes the same plan
/// (`archi/requirements/world-facts/a-plan-s-own-scenarios-block-retires.md`).
#[test]
fn a_post_world_plan_on_a_tree_with_a_world_refuses_until_a_fact_covers_a_node() {
    let root = temp_project();
    // The tree opted into the world — one fact stands, over a node this plan
    // holds no task for. There is a world to be behind on.
    put_fact(
        &root,
        "tunnels-run-long",
        "Tunnels run long",
        "Auth",
        &["the tunnel ends"],
    );
    // The fact reaches `Auth` and `Store`; `Gate` is declared for the save.
    declare_internal(&root, &["Gate"]);
    ok(&root, &["version", "save", "-m", "first"]);
    ok(&root, &["plan", "use", "mvp"]);

    // The mint writes no `scenarios.md`; a block a person leaves in the
    // folder is read by nobody and deleted by nothing.
    assert!(!root.join("archi/plans/mvp/scenarios.md").exists());
    write_record(
        &root,
        "archi/plans/mvp/scenarios.md",
        "# Scenarios\n\n- a user logs in end to end\n",
    );
    ok(&root, &["plan", "task", "add", "Store", "--desc", "persist rows"]);
    curate(&root, "archi/plans/mvp/t1-store.md", "store-encrypted", "code/store.rs");
    ok(&root, &["plan", "start"]);
    declares(&root, 1, "t1", &[STORE_ENTRY]);
    ok(&root, &["plan", "next"]);

    // No fact covers the node the plan holds a task for: the close refuses.
    let (_, err) = fails(&root, &["plan", "next"]);
    assert!(err.contains("no world fact covers any node"), "{err}");
    assert_eq!(state_json(&root, "mvp")["state"], "started");

    // The same plan closes once one covering fact stands — and the plan's
    // own block never prints.
    put_fact(
        &root,
        "riders-lose-the-signal",
        "Riders lose the signal",
        "Store",
        &["the app opens with no network"],
    );
    let out = ok(&root, &["plan", "next"]);
    assert!(out.contains("riders-lose-the-signal#the app opens with no network"), "{out}");
    assert!(!out.contains("a user logs in end to end"), "{out}");
    ok(
        &root,
        &[
            "link",
            "add",
            "riders-lose-the-signal#the app opens with no network",
            "code/store.rs",
            "--kind",
            "indirect",
        ],
    );
    let out = ok(&root, &["plan", "next"]);
    assert!(out.contains("DONE"), "{out}");

    // The old block is on disk exactly as the person left it, and no
    // finding names it.
    assert_eq!(
        fs::read_to_string(root.join("archi/plans/mvp/scenarios.md")).unwrap(),
        "# Scenarios\n\n- a user logs in end to end\n"
    );
    let (_, out, _) = run(&root, &["check"]);
    assert!(!out.contains("scenarios.md"), "{out}");

    fs::remove_dir_all(&root).unwrap();
}

/// A tree that holds no world fact at all has not opted into the world, and a
/// plan on it is not behind on one: the refusal needs a world to refuse
/// against, so the empty block closes exactly as a pre-world plan's does
/// (`archi/requirements/world-facts/a-plan-s-own-scenarios-block-retires.md`,
/// `archi/requirements/world-facts/the-world-arrives-without-noise.md`).
#[test]
fn a_post_world_plan_on_a_tree_with_no_world_closes_without_a_refusal() {
    let root = temp_project();
    ok(&root, &["version", "save", "-m", "first"]);
    ok(&root, &["plan", "use", "mvp"]);

    // The plan carries the mark of the world; the tree carries no world —
    // `archi/world/` was never created, and no verb creates it here.
    assert_eq!(state_json(&root, "mvp")["minted_after_the_world"], true);
    assert!(!root.join("archi/world").exists());

    ok(&root, &["plan", "task", "add", "Store", "--desc", "persist rows"]);
    curate(&root, "archi/plans/mvp/t1-store.md", "store-encrypted", "code/store.rs");
    ok(&root, &["plan", "start"]);
    declares(&root, 1, "t1", &[STORE_ENTRY]);
    let out = ok(&root, &["plan", "next"]);
    assert!(out.contains("the cleanup wave"), "{out}");

    // Nothing to be behind on: the close asks the world nothing and latches.
    let out = ok(&root, &["plan", "next"]);
    assert!(out.contains("DONE"), "{out}");
    assert!(!out.contains("no world fact covers any node"), "{out}");
    assert_eq!(state_json(&root, "mvp")["state"], "completed");
    assert!(!root.join("archi/world").exists());

    fs::remove_dir_all(&root).unwrap();
}

/// The closing step hands back the work it already did: every collected
/// scenario whole — its name and every step — with the state of its link
/// beside it. A scenario nothing anchors carries a ready `archi link add`;
/// one something anchors names the file and symbol it reaches and asks for
/// the re-read, and names the side that moved once the digests disagree
/// (`archi/requirements/world-facts/the-closing-step-hands-back-the-work.md`).
#[test]
fn the_closing_step_prints_the_gherkin_the_state_and_the_command() {
    let root = temp_project();
    put_fact_with_steps(
        &root,
        "riders-lose-the-signal",
        "Riders lose the signal",
        "Store",
        &[(
            "the app opens with no network",
            &[
                "Given the rider boards",
                "When the app opens",
                "Then the rows are there",
            ],
        )],
    );
    put_fact_with_steps(
        &root,
        "tunnels-run-long",
        "Tunnels run long",
        "Auth",
        &[(
            "the tunnel ends",
            &["Given the train is under the hill", "Then the session holds"],
        )],
    );
    // The two facts reach `Auth` and `Store`; `Gate` is declared for the save.
    declare_internal(&root, &["Gate"]);
    ok(&root, &["version", "save", "-m", "first"]);
    ok(&root, &["plan", "use", "mvp"]);
    ok(&root, &["plan", "task", "add", "Store", "--desc", "persist rows"]);
    ok(&root, &["plan", "task", "add", "Auth", "--desc", "guard the door"]);
    curate(&root, "archi/plans/mvp/t1-store.md", "store-encrypted", "code/store.rs");
    curate(&root, "archi/plans/mvp/t2-auth.md", "service-hardening", "code/auth.rs");

    // One of the two is anchored before the close, at a symbol inside a
    // file; the other is not.
    ok(
        &root,
        &[
            "link",
            "add",
            "riders-lose-the-signal#the app opens with no network",
            "code/store.rs#Store::put",
            "--kind",
            "indirect",
        ],
    );
    ok(&root, &["plan", "start"]);
    declares(&root, 1, "t1", &[STORE_ENTRY]);
    declares(&root, 1, "t2", &[AUTH_ENTRY]);
    ok(&root, &["plan", "next"]);
    let out = ok(&root, &["plan", "next"]);

    // The Gherkin whole: every step of both, under the address that names
    // them. The fact's own title is the feature, so no line says it again.
    assert!(out.contains("    Given the rider boards"), "{out}");
    assert!(out.contains("    When the app opens"), "{out}");
    assert!(out.contains("    Then the rows are there"), "{out}");
    assert!(out.contains("    Given the train is under the hill"), "{out}");
    assert!(out.contains("    Then the session holds"), "{out}");
    assert!(!out.contains("Feature:"), "{out}");

    // The anchored one names the file and the symbol it reaches, and asks
    // for the re-read: a link says the pair has not moved, never that the
    // two still say the same thing.
    assert!(
        out.contains(
            "riders-lose-the-signal#the app opens with no network — \
             anchored at code/store.rs#Store::put"
        ),
        "{out}"
    );
    assert!(
        out.contains("    read this scenario and that code against each other"),
        "{out}"
    );
    // The unanchored one carries the line that anchors it, ref quoted for a
    // shell, the code side left to the operator — and no ask, because there
    // is nothing yet to read it against.
    assert!(out.contains("tunnels-run-long#the tunnel ends — unanchored"), "{out}");
    assert_eq!(
        printed_link_add(&out),
        "archi link add 'tunnels-run-long#the tunnel ends' <file#symbol> --kind indirect"
    );

    // A step reworded under an anchored scenario parts the witness: the
    // anchor stays beside the side that moved, and the ask stays with it —
    // the listing answers the same way.
    put_fact_with_steps(
        &root,
        "riders-lose-the-signal",
        "Riders lose the signal",
        "Store",
        &[(
            "the app opens with no network",
            &[
                "Given the rider boards the carriage",
                "When the app opens",
                "Then the rows are there",
            ],
        )],
    );
    let listed = ok(&root, &["plan", "scenarios", "list"]);
    assert!(
        listed.contains(
            "riders-lose-the-signal#the app opens with no network — \
             anchored at code/store.rs#Store::put, drifted: the scenario side moved"
        ),
        "{listed}"
    );
    assert!(listed.contains("    Given the rider boards the carriage"), "{listed}");
    assert!(
        listed.contains("    read this scenario and that code against each other"),
        "{listed}"
    );

    fs::remove_dir_all(&root).unwrap();
}

/// The printed line is the product: a real `sh` reads it as written and the
/// scenario is anchored — the quoting is what makes the render worth more
/// than the name it replaced
/// (`archi/requirements/world-facts/the-closing-step-hands-back-the-work.md`).
#[test]
fn the_printed_link_add_line_anchors_the_scenario_through_a_real_shell() {
    let root = temp_project();
    // A name with a space and an apostrophe: the two things a hand-quoted
    // ref gets wrong.
    let name = "the rider's app opens with no network";
    put_fact_with_steps(
        &root,
        "riders-lose-the-signal",
        "Riders lose the signal",
        "Store",
        &[(name, &["Given the rider boards", "Then the rows are there"])],
    );
    // The fact covers `Store`; the nodes above it are declared for the save.
    declare_internal(&root, &["Gate", "Auth"]);
    ok(&root, &["version", "save", "-m", "first"]);
    ok(&root, &["plan", "use", "mvp"]);
    ok(&root, &["plan", "task", "add", "Store", "--desc", "persist rows"]);
    curate(&root, "archi/plans/mvp/t1-store.md", "store-encrypted", "code/store.rs");
    ok(&root, &["plan", "start"]);
    declares(&root, 1, "t1", &[STORE_ENTRY]);
    ok(&root, &["plan", "next"]);
    let out = ok(&root, &["plan", "next"]);

    let line = printed_link_add(&out);
    assert!(
        line.contains("'riders-lose-the-signal#the rider'\\''s app opens with no network'"),
        "the apostrophe closes the quote and is escaped: {line}"
    );

    // The operator supplies only the code side; everything else runs as
    // printed, through a shell that has never heard of this scenario.
    let command = line.replace("<file#symbol>", "code/store.rs");
    let bin = shim(&root);
    let (code, stdout, stderr) = shell(&bin, &command);
    assert_eq!(code, Some(0), "`{command}`:\n{stdout}\n{stderr}");

    // The latch is satisfied by what the shell did: the plan closes and the
    // scenario is named nowhere.
    let out = ok(&root, &["plan", "next"]);
    assert!(out.contains("DONE"), "{out}");
    assert!(!out.contains(name), "{out}");
    assert_eq!(state_json(&root, "mvp")["state"], "completed");

    fs::remove_dir_all(&root).unwrap();
}

/// `plan verify` answers with the same three states on demand, while a wave
/// is still open — the operator never has to reach the closing step to see
/// where the block stands
/// (`archi/requirements/world-facts/the-closing-step-hands-back-the-work.md`).
#[test]
fn plan_verify_prints_the_scenario_states_while_a_wave_is_still_open() {
    let root = temp_project();
    put_fact_with_steps(
        &root,
        "riders-lose-the-signal",
        "Riders lose the signal",
        "Store",
        &[
            ("the app opens with no network", &["Given the rider boards"]),
            ("the rider signs in", &["Given the rider boards"]),
            ("the tunnel ends", &["Given the rider boards"]),
        ],
    );
    // The fact covers `Store`; the nodes above it are declared for the save.
    declare_internal(&root, &["Gate", "Auth"]);
    ok(&root, &["version", "save", "-m", "first"]);
    ok(&root, &["plan", "use", "mvp"]);
    ok(&root, &["plan", "task", "add", "Store", "--desc", "persist rows"]);
    curate(&root, "archi/plans/mvp/t1-store.md", "store-encrypted", "code/store.rs");

    // One clean, one whose code moved under it, one nothing reaches.
    ok(
        &root,
        &[
            "link",
            "add",
            "riders-lose-the-signal#the rider signs in",
            "code/auth.rs#login",
            "--kind",
            "indirect",
        ],
    );
    ok(
        &root,
        &[
            "link",
            "add",
            "riders-lose-the-signal#the tunnel ends",
            "code/store.rs#Store::put",
            "--kind",
            "indirect",
        ],
    );
    ok(&root, &["plan", "start"]);
    fs::write(
        root.join("code/store.rs"),
        "pub struct Store;\nimpl Store {\n    pub fn put(&mut self, n: u8) { let _ = n; }\n}\n",
    )
    .unwrap();

    // Wave 1 is in flight — nothing closed, nothing latched.
    assert!(ok(&root, &["plan", "current-wave"]).contains("wave 1 in flight"));
    let out = ok(&root, &["plan", "verify"]);
    assert!(
        out.contains("scenario: riders-lose-the-signal#the app opens with no network — unanchored"),
        "{out}"
    );
    assert!(
        out.contains(
            "scenario: riders-lose-the-signal#the rider signs in — \
             anchored at code/auth.rs#login"
        ),
        "{out}"
    );
    assert!(
        out.contains(
            "scenario: riders-lose-the-signal#the tunnel ends — \
             anchored at code/store.rs#Store::put, drifted: the code side moved"
        ),
        "{out}"
    );
    assert!(
        out.contains("    read this scenario and that code against each other"),
        "{out}"
    );
    assert!(out.contains("    Given the rider boards"), "{out}");
    assert_eq!(
        printed_link_add(&out),
        "archi link add 'riders-lose-the-signal#the app opens with no network' \
         <file#symbol> --kind indirect"
    );

    fs::remove_dir_all(&root).unwrap();
}

/// A plan from before the world carries no mark, closes with no block, and
/// keeps the `scenarios.md` it was written with — history is left exactly
/// as it is (`archi/decisions/the-old-plans-are-left-alone.md`).
#[test]
fn a_pre_world_plan_closes_with_no_block_and_keeps_its_old_one() {
    let root = temp_project();
    put_fact(
        &root,
        "riders-lose-the-signal",
        "Riders lose the signal",
        "Auth",
        &["the app opens with no network"],
    );
    // The fact reaches `Auth` and `Store`; `Gate` is declared for the save.
    declare_internal(&root, &["Gate"]);
    ok(&root, &["version", "save", "-m", "first"]);
    ok(&root, &["plan", "use", "old"]);
    ok(&root, &["plan", "task", "add", "Store", "--desc", "persist rows"]);
    curate(&root, "archi/plans/old/t1-store.md", "store-encrypted", "code/store.rs");
    write_record(
        &root,
        "archi/plans/old/scenarios.md",
        "# Scenarios\n\n- a row survives a restart\n",
    );

    // The lifecycle file an older binary wrote: waves closed, no mark of
    // the world on it — the plan is what it was written as.
    let created = state_json(&root, "old")["created"]
        .as_str()
        .unwrap()
        .to_string();
    write_record(
        &root,
        "archi/plans/old/state.json",
        &format!(
            "{{\n  \"state\": \"started\",\n  \"closed_waves\": 1,\n  \
             \"version\": \"v0001\",\n  \"created\": \"{created}\"\n}}\n"
        ),
    );

    let out = ok(&root, &["plan", "next"]);
    assert!(out.contains("the cleanup wave"), "{out}");
    let out = ok(&root, &["plan", "next"]);
    assert!(out.contains("DONE"), "{out}");
    assert!(!out.contains("a row survives a restart"), "{out}");
    assert_eq!(state_json(&root, "old")["state"], "completed");
    assert_eq!(
        fs::read_to_string(root.join("archi/plans/old/scenarios.md")).unwrap(),
        "# Scenarios\n\n- a row survives a restart\n"
    );

    fs::remove_dir_all(&root).unwrap();
}

// ---- the declaration gates ---------------------------------------------------
//
// A wave does not close while a task in flight declared nothing, or while a
// declared claim the wave moved still stands on the code as it was
// (`archi/requirements/planning/an-undeclared-change-refuses-the-wave.md`,
// `archi/requirements/code-link/a-drifted-declaration-refuses-the-wave-that-moved-it.md`).

/// The tests the declarations below name. The file is written before the
/// plan starts, so it sits in every wave-open index and is never a change of
/// its own.
const TESTS_RS: &str = "pub fn a_row_is_persisted() {\n    assert!(true);\n}\n\n\
                        pub fn a_login_without_a_name_is_refused() {\n    assert!(true);\n}\n";

/// The proof a declaration over `code/store.rs` names.
const PROOF: &str = "code/tests.rs#a_row_is_persisted";

/// The proof a declaration over `code/auth.rs` names.
const AUTH_PROOF: &str = "code/tests.rs#a_login_without_a_name_is_refused";

/// `code/store.rs` as wave 1 leaves it: two symbols, both moved.
const STORE_TWO: &str = "pub struct Store;\nimpl Store {\n    \
                         pub fn put(&mut self, n: u8) { let _ = n; }\n    \
                         pub fn get(&self) -> u8 { 0 }\n}\n";

/// The same two symbols with both shapes moved again.
const STORE_TWO_MOVED: &str = "pub struct Store;\nimpl Store {\n    \
                               pub fn put(&mut self, n: u16) -> bool { let _ = n; true }\n    \
                               pub fn get(&self, k: u8) -> u16 { let _ = k; 0 }\n}\n";

/// The same two symbols with one shape moved and the other left standing.
const STORE_PUT_MOVED: &str = "pub struct Store;\nimpl Store {\n    \
                               pub fn put(&mut self, n: u16) -> bool { let _ = n; true }\n    \
                               pub fn get(&self) -> u8 { 0 }\n}\n";

/// The project-relative path of one task's declaration file in a wave of
/// plan `mvp`.
fn declares_rel(wave: usize, task: &str) -> String {
    format!("archi/plans/mvp/waves/w{wave:02}.{task}.declares.toml")
}

/// The t1 Store record the tests below drive. Its `Spec` names the node
/// alone — [`T1_STORE_CURATED`] carries the incoming edge beside it — so the
/// declaration gates are what these tests meet, and no edge ref presses the
/// coverage gate in front of them.
const T1_STORE_GATED: &str =
    "---\nnode: Store\nowns: [store-encrypted]\n---\n\n# t1 — Store\n\npersist rows\n\n\
     ## Spec\n\n- `Store`\n\n## Inputs\n\n## Outputs\n\n- code/store.rs\n\n## Stack\n\n\
     ## Verifications\n\n### store-encrypted\n\n- test — proves store-encrypted\n";

/// The t2 Auth record beside it. `outputs` is the block under `## Outputs`,
/// which is the whole of what varies: one test has t2 claim the file t1 wrote.
fn t2_auth_gated(outputs: &str) -> String {
    format!(
        "---\nnode: Auth\nowns: [service-hardening]\n---\n\n# t2 — Auth\n\nguard the door\n\n\
         ## Spec\n\n- `Auth`\n- `Service type_of Auth`\n\n## Inputs\n\n\
         - from t1 — the store api\n\n## Outputs\n\n{outputs}\n## Stack\n\n## Verifications\n\n\
         ### service-hardening\n\n- test — proves service-hardening\n"
    )
}

/// The one declaration a `Store` task writes when its own work is not the
/// subject of the test: one pair, which is all the gate asks for.
const STORE_ENTRY: [&str; 3] = ["code/store.rs#Store::put", "Store", PROOF];

/// The same for an `Auth` task.
const AUTH_ENTRY: [&str; 3] = ["code/auth.rs#login", "Auth", AUTH_PROOF];

/// Write one task's declaration file, as its sub-agent does before it
/// returns: one `[[declares]]` table per pair it accounts for. An empty
/// list writes the file that names nothing, which the wave refuses.
fn declares(root: &Path, wave: usize, task: &str, entries: &[[&str; 3]]) {
    let mut text = String::new();
    for [symbol, answers, proved_by] in entries {
        text.push_str(&format!(
            "[[declares]]\nsymbol = \"{symbol}\"\nanswers = \"{answers}\"\n\
             proved_by = \"{proved_by}\"\n\n"
        ));
    }
    if entries.is_empty() {
        text.push_str("declares = []\n");
    }
    let path = root.join(declares_rel(wave, task));
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

/// The id of the one live link whose row holds `row`.
fn link_id(root: &Path, row: &str) -> String {
    let rows = ok(root, &["link", "ls"]);
    let hits: Vec<&str> = rows.lines().filter(|l| l.contains(row)).collect();
    assert_eq!(hits.len(), 1, "one row holds `{row}`:\n{rows}");
    hits[0]
        .split_whitespace()
        .next()
        .expect("a row starts with its id")
        .to_string()
}

/// Age one journaled row into the shape a row written before the rule field
/// existed carries: no rule at all, which reads back as `inferred` from its
/// captured origin — the shape the standing rows of a real tree carry
/// (`archi/requirements/code-link/the-journal-says-which-rule-made-a-row.md`).
fn unstamp_rule(root: &Path, id: &str) {
    let path = root.join("archi/links/journal.jsonl");
    let text = fs::read_to_string(&path).unwrap();
    let aged: Vec<String> = text
        .lines()
        .map(|l| {
            if l.contains(id) {
                l.replace(",\"rule\":\"declared\"", "")
            } else {
                l.to_string()
            }
        })
        .collect();
    fs::write(&path, format!("{}\n", aged.join("\n"))).unwrap();
}

/// The gate is the file and nothing else. A task in flight accounts for its
/// work by writing one; a file that names nothing accounts for nothing and
/// refuses the same way. What the file holds is the writer's to decide — one
/// entry closes the wave, whatever else the delta moved
/// (`archi/requirements/planning/an-undeclared-change-refuses-the-wave.md`).
#[test]
fn a_wave_refuses_until_every_task_in_flight_has_declared() {
    let root = temp_project();
    ok(&root, &["version", "save", "-m", "first"]);
    ok(&root, &["plan", "use", "mvp"]);
    ok(&root, &["plan", "task", "add", "Store"]);
    ok(&root, &["plan", "task", "add", "Auth"]);
    write_record(&root, "archi/plans/mvp/t1-store.md", T1_STORE_GATED);
    write_record(
        &root,
        "archi/plans/mvp/t2-auth.md",
        &t2_auth_gated("- code/auth.rs\n"),
    );
    ok(&root, &["plan", "start"]);

    // Two symbols move under t1's one output, and a file no task claims moves
    // beside them.
    fs::write(root.join("code/store.rs"), STORE_TWO).unwrap();
    fs::write(root.join("code/orphan.rs"), "pub fn stray() -> u8 { 7 }\n").unwrap();

    // No file: the refusal names the task, the path it owes and the command
    // that follows.
    let (stdout, err) = fails(&root, &["plan", "next"]);
    assert!(err.contains("t1 — write"), "names the task: {err}");
    assert!(err.contains(&declares_rel(1, "t1")), "names the path: {err}");
    assert!(err.contains("re-run `archi plan next`"), "names the next command: {err}");
    assert!(stdout.contains("leftover code/orphan.rs#stray"), "{stdout}");

    // A file that parses and declares nothing accounts for nothing: the same
    // refusal, naming the same task, the same path and the same command.
    declares(&root, 1, "t1", &[]);
    let (_, err) = fails(&root, &["plan", "next"]);
    assert!(err.contains("names nothing"), "{err}");
    assert!(err.contains("t1"), "names the task: {err}");
    assert!(err.contains(&declares_rel(1, "t1")), "names the path: {err}");
    assert!(err.contains("re-run `archi plan next`"), "names the next command: {err}");

    // One entry closes the wave, whatever else the delta moved: two symbols
    // moved under t1's output and the file names one of them.
    declares(&root, 1, "t1", &[STORE_ENTRY]);
    let out = ok(&root, &["plan", "next"]);
    assert!(out.contains("wave 1 closed — in flight: t2"), "{out}");
    assert!(out.contains("leftover code/orphan.rs#stray"), "{out}");

    // The same gate on the next wave, and the same one line answers it.
    let (_, err) = fails(&root, &["plan", "next"]);
    assert!(err.contains("t2 — write"), "{err}");
    declares(&root, 2, "t2", &[AUTH_ENTRY]);
    let out = ok(&root, &["plan", "next"]);
    assert!(out.contains("the cleanup wave"), "{out}");

    // Past the last wave no task is in flight, so no file is owed: the
    // cleanup step and the close run with none on disk.
    assert!(!root.join(declares_rel(3, "t1")).exists());
    let out = ok(&root, &["plan", "next"]);
    assert!(out.contains("DONE"), "{out}");

    fs::remove_dir_all(&root).unwrap();
}

/// A declared pair the wave moves out from under refuses that wave, and the
/// refusal names the link, the symbol and both exits. The rows nobody
/// declared — inferred and hand-authored — drift beside it and say nothing
/// (`archi/requirements/code-link/a-drifted-declaration-refuses-the-wave-that-moved-it.md`).
#[test]
fn a_wave_that_moves_a_declared_symbol_refuses_until_the_pair_is_repinned() {
    let root = temp_project();
    ok(&root, &["version", "save", "-m", "first"]);
    ok(&root, &["plan", "use", "mvp"]);
    ok(&root, &["plan", "task", "add", "Store"]);
    ok(&root, &["plan", "task", "add", "Auth"]);
    ok(&root, &["plan", "task", "add", "Gate"]);
    write_record(&root, "archi/plans/mvp/t1-store.md", T1_STORE_GATED);
    write_record(
        &root,
        "archi/plans/mvp/t2-auth.md",
        &t2_auth_gated("- code/auth.rs\n"),
    );
    write_record(
        &root,
        "archi/plans/mvp/t3-gate.md",
        "---\nnode: Gate\nowns: []\n---\n\n# t3 — Gate\n\nopen the door\n\n\
         ## Spec\n\n- `Gate`\n\n## Inputs\n\n- from t2 — the guard\n\n\
         ## Outputs\n\n- code/gate.rs\n- code/store.rs\n\n## Stack\n\n## Verifications\n",
    );
    ok(&root, &["plan", "start"]);

    // Wave 1: two symbols move and t1 declares both.
    fs::write(root.join("code/store.rs"), STORE_TWO).unwrap();
    declares(
        &root,
        1,
        "t1",
        &[
            ["code/store.rs#Store::put", "Store", PROOF],
            ["code/store.rs#Store::get", "Store", PROOF],
        ],
    );
    let out = ok(&root, &["plan", "next"]);
    assert!(out.contains("wave 1 closed — in flight: t2"), "{out}");
    let declared = link_id(&root, "Store ← code/store.rs#Store::put");
    let aged = link_id(&root, "Store ← code/store.rs#Store::get");
    unstamp_rule(&root, &aged);

    // Wave 2 moves nothing in `code/store.rs`: the declared pair stands and
    // the wave closes.
    fs::write(
        root.join("code/auth.rs"),
        "pub fn login(u: &str) -> bool { !u.is_empty() }\n",
    )
    .unwrap();
    declares(&root, 2, "t2", &[["code/auth.rs#login", "Auth", AUTH_PROOF]]);
    let out = ok(&root, &["plan", "next"]);
    assert!(out.contains("wave 2 closed — in flight: t3"), "{out}");

    // A hand-authored row on the same symbol, to drift beside the declared one.
    ok(&root, &["link", "add", "Gate", "code/store.rs#Store::put", "--kind", "indirect"]);
    let authored = link_id(&root, "Gate ← code/store.rs#Store::put");

    // Wave 3 moves both symbols. t3 declares them again — the same claim,
    // so nothing new is minted — and the declared pair it moved refuses.
    fs::write(root.join("code/store.rs"), STORE_TWO_MOVED).unwrap();
    declares(
        &root,
        3,
        "t3",
        &[
            ["code/store.rs#Store::put", "Store", PROOF],
            ["code/store.rs#Store::get", "Store", PROOF],
        ],
    );
    let (_, err) = fails(&root, &["plan", "next"]);
    assert!(err.contains(&declared), "names the link: {err}");
    assert!(err.contains("code/store.rs#Store::put"), "names the symbol: {err}");
    assert!(
        err.contains(&format!("archi link repin {declared}")),
        "the first exit: {err}"
    );
    assert!(
        err.contains(&format!("archi link rm {declared}")),
        "the second exit: {err}"
    );
    assert!(!err.contains(&aged), "an inferred row that drifts says nothing: {err}");
    assert!(
        !err.contains(&authored),
        "a hand-authored row that drifts says nothing: {err}"
    );

    // Repinning accepts the drift, and the wave closes.
    ok(&root, &["link", "repin", &declared]);
    let out = ok(&root, &["plan", "next"]);
    assert!(out.contains("the cleanup wave"), "{out}");

    fs::remove_dir_all(&root).unwrap();
}

/// The second exit the drift refusal names: the claim no longer holds, so the
/// stale pair is retired and this wave's declaration mints it again against
/// the code as it stands now. The file is the claim, so the pair returns —
/// which is what the refusal says it will do
/// (`archi/requirements/code-link/a-drifted-declaration-refuses-the-wave-that-moved-it.md`).
#[test]
fn the_second_exit_retires_the_stale_pair_and_the_declaration_mints_it_anew() {
    let root = temp_project();
    ok(&root, &["version", "save", "-m", "first"]);
    ok(&root, &["plan", "use", "mvp"]);
    ok(&root, &["plan", "task", "add", "Store"]);
    ok(&root, &["plan", "task", "add", "Auth"]);
    write_record(&root, "archi/plans/mvp/t1-store.md", T1_STORE_GATED);
    write_record(
        &root,
        "archi/plans/mvp/t2-auth.md",
        &t2_auth_gated("- code/auth.rs\n- code/store.rs\n"),
    );
    ok(&root, &["plan", "start"]);

    fs::write(root.join("code/store.rs"), STORE_TWO).unwrap();
    declares(
        &root,
        1,
        "t1",
        &[
            ["code/store.rs#Store::put", "Store", PROOF],
            ["code/store.rs#Store::get", "Store", PROOF],
        ],
    );
    ok(&root, &["plan", "next"]);
    let declared = link_id(&root, "Store ← code/store.rs#Store::put");

    // Wave 2 moves the symbol t1 declared, and t2 claims the file it sits in.
    fs::write(root.join("code/store.rs"), STORE_PUT_MOVED).unwrap();
    declares(&root, 2, "t2", &[["code/store.rs#Store::put", "Store", PROOF]]);
    let (_, err) = fails(&root, &["plan", "next"]);
    assert!(err.contains(&format!("archi link rm {declared}")), "{err}");

    // The claim did not hold: retire it, and the file that declares the
    // symbol mints the pair again — this time under the task that moved it.
    ok(&root, &["link", "rm", &declared]);
    let out = ok(&root, &["plan", "next"]);
    assert!(out.contains("the cleanup wave"), "{out}");
    let rows = ok(&root, &["link", "ls"]);
    let put: Vec<&str> = rows
        .lines()
        .filter(|l| l.contains("← code/store.rs#Store::put"))
        .collect();
    assert_eq!(put.len(), 1, "the retired pair came back once:\n{rows}");
    assert!(put[0].contains("captured(t2)") && put[0].contains("declared"), "{rows}");

    fs::remove_dir_all(&root).unwrap();
}
