//! End to end through the real binary: the `world` verb
//! (`archi/requirements/world-facts/`). `add` and `rm` mutate, so they meet
//! the same seat rule and the same exit codes as `req add|rm`; `ls` reads
//! and answers anywhere. `--covers <element>` is the traversal the wing is
//! reached by (`archi/decisions/the-wing-is-reached-by-traversal.md`), and
//! an empty answer on either retrieval path names the other one.

mod util;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::{Value, json};

const MODEL: &str = "\
def node AuthService:
  port handle_login
def node RateLimiter:
  port take
def node Island
def conn calls := * -> *
AuthService.handle_login calls RateLimiter.take
";

/// The scaffolded project and the worktree minted from it. The primary
/// checkout stays unbound — the case every refusal answers — and the
/// worktree is where the mutations run.
fn temp_project() -> (PathBuf, PathBuf) {
    let dir = util::scratch("archi-world-e2e", "p");
    fs::create_dir_all(dir.join("archi/src")).unwrap();
    fs::write(
        dir.join("archi.toml"),
        "[project]\nname = \"t\"\npreset = \"default\"\n",
    )
    .unwrap();
    fs::write(dir.join("archi/src/model.arch"), MODEL).unwrap();
    let intent = dir.join("archi/requirements/hardening");
    fs::create_dir_all(&intent).unwrap();
    fs::write(
        intent.join("hardening.md"),
        "# Hardening\n\nThe area under pressure.\n",
    )
    .unwrap();
    let wt = util::worktree(&dir);
    (dir, wt)
}

fn put(root: &Path, rel_path: &str, text: &str) {
    let path = root.join(rel_path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

/// One whole world fact: the three lists as given, the conditioning
/// paragraph, the workaround and one scenario.
fn fact(root: &Path, slug: &str, title: &str, covers: &str, sources: &str, uses: &str) {
    util::Fact {
        covers,
        sources,
        uses,
        condition: "The carriage drops the network for minutes at a time.",
        workaround: "Riders load the page at the platform and redo the trip's work when they \
                     forget.",
        scenarios: "### the app opens with no network\n\n\
                    Given the device has no network\nWhen the user opens the app\n\
                    Then the last synced view appears\n",
    }
    .write(root, slug, title);
}

/// The nodes of [`MODEL`] the facts of a test never reach: the shared
/// declaration ([`util::declare_internal`]), for the tests here that only need
/// a version to exist.
fn declare_internal(root: &Path, nodes: &[&str]) {
    util::declare_internal(root, nodes);
}

/// The note every grounded fact here rests on, in the layer a source lives in
/// (`archi/requirements/world-facts/a-source-is-reachable-and-lives-in-the-world.md`).
const NOTE: &str = "archi/world/notes/line.md";

/// The note [`NOTE`] names: a name and the prose under it, which is the whole
/// schema of a loose layer.
fn note(root: &Path) {
    put(root, NOTE, "# The line\n\nThe ride, written down.\n");
}

/// Three standing facts: one on the gate, one on both, one on the limiter —
/// and one of them ungrounded.
fn three_facts(root: &Path) {
    note(root);
    fact(
        root,
        "trains-lose-the-signal",
        "Trains lose the signal",
        "AuthService",
        NOTE,
        "",
    );
    fact(
        root,
        "tunnels-run-long",
        "Tunnels run long",
        "AuthService, RateLimiter",
        "",
        "trains-lose-the-signal",
    );
    fact(
        root,
        "the-guard-walks-the-line",
        "The guard walks the line",
        "RateLimiter",
        NOTE,
        "",
    );
}

/// The binary's exit code, stdout and stderr.
fn run(root: &Path, args: &[&str]) -> (Option<i32>, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_archi"))
        .args(args)
        .args(["--project", root.to_str().unwrap()])
        .output()
        .expect("archi runs");
    (
        out.status.code(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

fn ok(root: &Path, args: &[&str]) -> String {
    let (code, stdout, stderr) = run(root, args);
    assert_eq!(code, Some(0), "archi {args:?} failed:\n{stdout}\n{stderr}");
    stdout
}

/// A refusal: its exit code and its message.
fn refuse(root: &Path, args: &[&str]) -> (Option<i32>, String) {
    let (code, stdout, stderr) = run(root, args);
    assert_ne!(code, Some(0), "archi {args:?} passed but had to refuse:\n{stdout}");
    (code, stderr)
}

fn parsed(text: &str) -> Value {
    serde_json::from_str(text).unwrap_or_else(|e| panic!("json: {e}\n{text}"))
}

/// This family's shim: the shared one ([`util::shim`]) under this family's
/// own scratch name.
fn shim(root: &Path) -> PathBuf {
    util::shim(root, "archi-world-e2e")
}

/// Run one line through `sh`, exactly as it was printed ([`util::shell`]).
fn shell(bin: &Path, line: &str) -> (Option<i32>, String, String) {
    util::shell(bin, line)
}

/// The command lines of a refusal — every line the head line does not carry.
fn commands(refusal: &str) -> Vec<String> {
    refusal
        .lines()
        .skip(1)
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect()
}

/// The whole first flow, end to end: the operator mints the skeleton, fills
/// the prose, and `check` holds the file the whole way — first for the slots
/// nobody wrote, then for the references that resolve to nothing — and lets
/// go the moment the record is whole
/// (`archi/requirements/world-facts/one-verb-mints-the-world-fact.md`,
/// `archi/requirements/world-facts/the-header-points-three-ways.md`).
#[test]
fn the_check_holds_the_minted_fact_until_it_is_whole() {
    let (_primary, wt) = temp_project();
    ok(&wt, &["world", "add", "Trains lose the signal"]);

    // The skeleton alone: one located error per empty slot, and the check
    // refuses — the mint hands back a worklist, not a finished record.
    let (code, _out, err) = run(&wt, &["check"]);
    assert_eq!(code, Some(1), "{err}");
    assert_eq!(
        err.matches("archi/world/facts/trains-lose-the-signal.md")
            .count(),
        3,
        "{err}"
    );
    assert!(err.contains("a world fact needs a summary paragraph"), "{err}");
    assert!(err.contains("`What people do instead` holds nothing"), "{err}");
    assert!(err.contains("`Scenarios` holds nothing"), "{err}");

    // The prose lands, but the three lists point at nothing: an element no
    // model declares, a file no layer of the world holds, a fact no wing
    // holds.
    fact(
        &wt,
        "trains-lose-the-signal",
        "Trains lose the signal",
        "Gate",
        "archi/world/notes/ghost.md",
        "tunnels-run-long",
    );
    let (code, _out, err) = run(&wt, &["check"]);
    assert_eq!(code, Some(1), "{err}");
    assert!(err.contains("covers names no element `Gate`"), "{err}");
    assert!(
        err.contains("`sources` names no file `archi/world/notes/ghost.md`"),
        "{err}"
    );
    assert!(
        err.contains("uses names no world fact `tunnels-run-long`"),
        "{err}"
    );
    // The slots are written now, and nothing says otherwise.
    assert!(!err.contains("holds nothing"), "{err}");

    // Every reference resolves: the element the model declares, a note that
    // stands in the world, and no dependency at all.
    note(&wt);
    fact(
        &wt,
        "trains-lose-the-signal",
        "Trains lose the signal",
        "AuthService",
        NOTE,
        "",
    );
    let (code, out, err) = run(&wt, &["check"]);
    assert_eq!(code, Some(0), "{out}{err}");
    assert!(!err.contains("archi/world/"), "{err}");
    assert!(out.contains("world — 1 facts · 0 ungrounded"), "{out}");
}

/// The whole retirement flow: a fact a dependant, a link and an open plan
/// all hold refuses once, with the ordered commands that clear each hold —
/// and each of those lines, pasted into a shell as printed, clears its own
/// (`archi/requirements/world-facts/the-refusal-is-an-ordered-continuation.md`,
/// `archi/requirements/world-facts/removal-names-the-code-it-strands.md`,
/// `archi/requirements/world-facts/retirement-refuses-a-plan-in-flight.md`).
#[test]
fn the_held_removal_hands_back_the_commands_that_clear_it() {
    let (_primary, wt) = temp_project();
    note(&wt);
    put(&wt, "code/app.rs", "pub fn open() -> bool { true }\n");
    fact(
        &wt,
        "trains-lose-the-signal",
        "Trains lose the signal",
        "AuthService",
        NOTE,
        "",
    );
    // A second fact rests on it.
    fact(
        &wt,
        "tunnels-run-long",
        "Tunnels run long",
        "RateLimiter",
        NOTE,
        "trains-lose-the-signal",
    );
    // A link anchors one of its scenarios in code.
    ok(
        &wt,
        &[
            "link",
            "add",
            "trains-lose-the-signal#the app opens with no network",
            "code/app.rs",
            "--kind",
            "indirect",
        ],
    );
    // And a plan in flight carries it: the task on the node the fact covers.
    // The save now gates on the wing's reach, and `Island` is the node these
    // two facts never touch — it is declared internal so the gate lets this
    // test get to the plan it is about.
    declare_internal(&wt, &["Island"]);
    ok(&wt, &["version", "save", "-m", "first"]);
    ok(&wt, &["plan", "use", "offline-open"]);
    ok(&wt, &["plan", "task", "add", "AuthService", "--desc", "hold the door"]);
    assert!(
        ok(&wt, &["plan", "task", "show", "t1"]).contains("fact: trains-lose-the-signal"),
        "the plan carries the fact"
    );

    // One refusal for all three, and nothing retired.
    let (code, err) = refuse(&wt, &["world", "rm", "trains-lose-the-signal"]);
    assert_eq!(code, Some(1));
    let head = err.lines().next().expect("a head line");
    assert!(head.contains("1 plan in flight"), "{err}");
    assert!(head.contains("1 stranded link"), "{err}");
    assert!(head.contains("1 dependant fact"), "{err}");
    let lines = commands(&err);
    assert_eq!(lines.len(), 3, "{err}");
    assert!(
        lines[0].starts_with("archi plan use offline-open && archi plan close"),
        "{err}"
    );
    assert!(
        lines[1].starts_with(
            "archi link rm --spec 'trains-lose-the-signal#the app opens with no network' --yes"
        ),
        "{err}"
    );
    assert!(lines[2].starts_with("archi world rm tunnels-run-long"), "{err}");
    assert!(err.contains("code/app.rs"), "the line names the code it strands: {err}");
    assert!(wt.join("archi/world/facts/trains-lose-the-signal.md").is_file());
    assert!(wt.join("archi/world/facts/tunnels-run-long.md").is_file());

    // Paste them into a shell in the printed order: each clears its own hold
    // and the removal is still held until the last one runs.
    let bin = shim(&wt);
    for (i, line) in lines.iter().enumerate() {
        let (code, out, err) = shell(&bin, line);
        assert_eq!(code, Some(0), "`{line}` did not run:\n{out}{err}");
        if i + 1 < lines.len() {
            let (_, err) = refuse(&wt, &["world", "rm", "trains-lose-the-signal"]);
            assert_eq!(commands(&err).len(), lines.len() - i - 1, "{err}");
        }
    }
    ok(&wt, &["world", "rm", "trains-lose-the-signal"]);
    assert!(!wt.join("archi/world/facts/trains-lose-the-signal.md").exists());
    // Nothing cascaded: the dependant retired by its own printed line, and
    // the wing is what those lines left behind.
    assert_eq!(ok(&wt, &["world", "ls"]), "");
}

/// `add` and `rm` mutate, so the seat rule holds for them exactly as it
/// holds for `req`; `ls` reads and runs anywhere
/// (`the-world-verb-refuses-like-the-others`).
#[test]
fn the_world_verb_refuses_like_the_others() {
    let (primary, wt) = temp_project();

    // The unbound checkout refuses the mint, and names the standing seat.
    let (code, err) = refuse(&primary, &["world", "add", "Trains lose the signal"]);
    assert!(err.contains("unbound"), "{err}");
    assert!(err.contains(wt.to_str().unwrap()), "{err}");

    // The same class of refusal as `req add`, down to the exit code.
    let (req_code, _) = refuse(
        &primary,
        &[
            "req", "add", "Gate throttles", "--intent", "hardening", "--kind", "functional",
            "--origin", "intent",
        ],
    );
    assert_eq!(code, req_code);

    // `rm` is a mutation too.
    let (rm_code, err) = refuse(&primary, &["world", "rm", "trains-lose-the-signal"]);
    assert_eq!(rm_code, code);
    assert!(err.contains("unbound"), "{err}");

    // The refusal left no file and no folder behind.
    assert!(!primary.join("archi/world").exists());

    // `ls` reads: it answers in the same unbound checkout, exit zero, and a
    // tree with no wing prints nothing.
    assert_eq!(ok(&primary, &["world", "ls"]), "");
}

/// The mint writes the shape the reader accepts: a person who fills the
/// skeleton's empty slots and touches no heading lands a fact the check lets
/// go (`archi/requirements/world-facts/one-verb-mints-the-world-fact.md`,
/// `archi/requirements/world-facts/a-world-fact-carries-its-scenarios.md`).
///
/// The mint is the one place the tool authors a heading, so a heading the
/// reader refuses would make `archi world add` write a file no amount of
/// prose can rescue — the operator would have to know to rename a section the
/// tool had just handed them. Nothing else in the fixture is edited: what is
/// written is prose under the headings the mint chose.
#[test]
fn the_minted_skeleton_takes_prose_and_passes_the_check() {
    let (_primary, wt) = temp_project();
    let out = ok(&wt, &["world", "add", "Trains lose the signal"]);
    // The line the mint prints names the slots, and the workaround is one.
    assert!(out.contains("what people do instead"), "{out}");

    let path = wt.join("archi/world/facts/trains-lose-the-signal.md");
    let minted = fs::read_to_string(&path).unwrap();
    // No heading of the minted file is touched: prose goes under each of
    // them, and the covers list takes the node it conditions.
    let filled = minted
        .replace("covers: []", "covers: [AuthService]")
        .replace(
            "# Trains lose the signal\n",
            "# Trains lose the signal\n\nThe carriage drops the network for minutes at a time.\n",
        )
        .replace(
            "## What people do instead\n",
            "## What people do instead\n\nRiders load the page at the platform and redo the \
             trip's work when they forget.\n",
        )
        .replace(
            "## Scenarios\n",
            "## Scenarios\n\n### the app opens with no network\n\n\
             Given the device has no network\nWhen the user opens the app\n\
             Then the last synced view appears\n",
        );
    assert_ne!(filled, minted);
    fs::write(&path, &filled).unwrap();

    let (code, out, err) = run(&wt, &["check"]);
    assert_eq!(code, Some(0), "{out}{err}");
    assert!(!err.contains("archi/world/facts/"), "{err}");
    assert!(out.contains("world — 1 facts"), "{out}");
}

/// The mint and the removal ride the same dispatch as the other doc verbs:
/// the skeleton, the refusals, the exit codes. The skeleton lands in the
/// layer of the strict record and nowhere else
/// (`archi/requirements/world-facts/the-world-holds-four-layers.md`).
#[test]
fn the_verb_mints_and_retires_from_the_bound_seat() {
    let (_primary, wt) = temp_project();

    let out = ok(&wt, &["world", "add", "Trains lose the signal"]);
    assert!(
        out.contains("archi/world/facts/trains-lose-the-signal.md"),
        "{out}"
    );
    let path = wt.join("archi/world/facts/trains-lose-the-signal.md");
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "---\ncovers: []\nsources: []\nuses: []\n---\n\n\
         # Trains lose the signal\n\n## What people do instead\n\n## Scenarios\n"
    );
    assert!(ok(&wt, &["world", "ls"]).contains("trains-lose-the-signal"));

    // A file whose prose was written is a wall, at the exit code every doc
    // verb refuses with.
    fact(&wt, "trains-lose-the-signal", "Trains lose the signal", "AuthService", "", "");
    let (code, err) = refuse(&wt, &["world", "add", "Trains lose the signal"]);
    assert_eq!(code, Some(1));
    assert!(
        err.contains("archi/world/facts/trains-lose-the-signal.md"),
        "{err}"
    );

    ok(&wt, &["world", "rm", "trains-lose-the-signal"]);
    assert!(!path.exists());
    let (code, err) = refuse(&wt, &["world", "rm", "trains-lose-the-signal"]);
    assert_eq!(code, Some(1));
    assert!(err.contains("no world fact"), "{err}");

    // A missing parameter is a usage refusal, before the tree is touched.
    let (code, err) = refuse(&wt, &["world", "add"]);
    assert_eq!(code, Some(2));
    assert!(err.contains("archi world add"), "{err}");
    assert!(
        !wt.join("archi/world/facts")
            .join("trains-lose-the-signal.md")
            .exists()
    );
}

/// The wing arrives with the file: a tree that holds no `archi/world/` at all
/// takes its first fact, and the mint makes the layer and the folder over it
/// on the way (`archi/requirements/world-facts/the-wing-arrives-without-noise.md`,
/// `archi/requirements/world-facts/the-world-holds-four-layers.md`).
#[test]
fn the_first_mint_creates_the_layer_and_its_parent() {
    let (_primary, wt) = temp_project();
    assert!(!wt.join("archi/world").exists(), "the tree opens with no wing");

    ok(&wt, &["world", "add", "Trains lose the signal"]);
    assert!(wt.join("archi/world/facts").is_dir());
    assert!(
        wt.join("archi/world/facts/trains-lose-the-signal.md")
            .is_file()
    );
    // Nothing stands in the wing's root: the folder is what says how a file
    // is read, so the mint leaves no file outside a layer.
    let loose: Vec<String> = fs::read_dir(wt.join("archi/world"))
        .unwrap()
        .flatten()
        .filter(|e| e.path().is_file())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(loose, Vec::<String>::new());

    // The walk that lists a fact finds it in its new home, and the removal
    // takes it from there.
    assert!(ok(&wt, &["world", "ls"]).contains("archi/world/facts/trains-lose-the-signal.md"));
    ok(&wt, &["world", "rm", "trains-lose-the-signal"]);
    assert!(
        !wt.join("archi/world/facts/trains-lose-the-signal.md")
            .exists()
    );
}

/// A replayed mint converges: the second `world add` on the untouched
/// skeleton reports `already minted`, leaves the file byte-identical, and
/// exits with the code `req add` returns for the same case. A file whose
/// prose was written is the other half — a wall that names it
/// (`archi/requirements/world-facts/one-verb-mints-the-world-fact.md`).
#[test]
fn the_replayed_mint_converges_like_req_add() {
    let (_primary, wt) = temp_project();
    let path = wt.join("archi/world/facts/trains-lose-the-signal.md");
    ok(&wt, &["world", "add", "Trains lose the signal"]);
    let minted = fs::read_to_string(&path).unwrap();

    let (code, out, err) = run(&wt, &["world", "add", "Trains lose the signal"]);
    assert!(out.contains("already minted"), "{out}{err}");
    assert_eq!(fs::read_to_string(&path).unwrap(), minted, "the mint rewrote the file");

    // The same case through `req add`: same words, same exit code.
    let req = [
        "req", "add", "Gate throttles", "--intent", "hardening", "--kind", "functional",
        "--origin", "intent",
    ];
    ok(&wt, &req);
    let (req_code, req_out, _) = run(&wt, &req);
    assert!(req_out.contains("already minted"), "{req_out}");
    assert_eq!(code, req_code);
    assert_eq!(code, Some(0), "{err}");

    // One authored line and the mint is a wall that names the file.
    fs::write(
        &path,
        minted.replace(
            "## Scenarios\n",
            "## Scenarios\n\n### the app opens with no network\n",
        ),
    )
    .unwrap();
    let (code, err) = refuse(&wt, &["world", "add", "Trains lose the signal"]);
    assert_eq!(code, Some(1));
    assert!(
        err.contains("archi/world/facts/trains-lose-the-signal.md"),
        "{err}"
    );
}

/// `world ls` lists the wing, and `--covers` walks the bridge
/// (`one-verb-walks-the-bridge`).
#[test]
fn one_verb_walks_the_bridge() {
    let (_primary, wt) = temp_project();
    three_facts(&wt);

    // One block per fact: slug, path, covers — and whether it is grounded.
    let out = ok(&wt, &["world", "ls"]);
    for slug in [
        "the-guard-walks-the-line",
        "trains-lose-the-signal",
        "tunnels-run-long",
    ] {
        assert!(
            out.contains(&format!("{slug}  archi/world/facts/{slug}.md")),
            "{out}"
        );
    }
    assert_eq!(out.matches("covers:").count(), 3, "{out}");
    assert!(out.contains("covers: AuthService, RateLimiter"), "{out}");
    assert!(out.contains("sources: none"), "{out}");
    assert!(out.contains("uses: trains-lose-the-signal"), "{out}");

    // `--covers` returns the facts naming that element, and only those.
    let out = ok(&wt, &["world", "ls", "--covers", "RateLimiter"]);
    assert!(out.contains("tunnels-run-long"), "{out}");
    assert!(out.contains("the-guard-walks-the-line"), "{out}");
    // The fact that covers the gate alone is not a block here — it rides
    // the listing only as the `uses` of one that is.
    assert!(
        !out.contains("trains-lose-the-signal  archi/world/facts/"),
        "{out}"
    );

    // A name no model element matches refuses and says so.
    let (code, err) = refuse(&wt, &["world", "ls", "--covers", "Ghost"]);
    assert_eq!(code, Some(1));
    assert!(err.contains("Ghost"), "{err}");
    assert!(err.contains("names no element"), "{err}");

    // The envelope carries the record: slug, path, covers, sources, uses.
    let v = parsed(&ok(&wt, &["world", "ls", "--json"]));
    assert_eq!(v["status"], "ok");
    let facts = v["facts"].as_array().unwrap();
    assert_eq!(facts.len(), 3);
    let f = facts
        .iter()
        .find(|f| f["slug"] == "tunnels-run-long")
        .expect("the fact");
    assert_eq!(f["path"], "archi/world/facts/tunnels-run-long.md");
    assert_eq!(f["covers"], json!(["AuthService", "RateLimiter"]));
    assert_eq!(f["sources"], json!([]));
    assert_eq!(f["uses"], json!(["trains-lose-the-signal"]));
    let f = facts
        .iter()
        .find(|f| f["slug"] == "the-guard-walks-the-line")
        .expect("the fact");
    assert_eq!(f["sources"], json!([NOTE]));

    // The filter narrows the envelope the same way.
    let v = parsed(&ok(&wt, &["world", "ls", "--covers", "AuthService", "--json"]));
    assert_eq!(v["facts"].as_array().unwrap().len(), 2);
}

/// An empty answer on either path names the other one
/// (`each-retrieval-path-names-the-other`).
#[test]
fn each_retrieval_path_names_the_other() {
    let (_primary, wt) = temp_project();
    three_facts(&wt);

    // No fact covers the island: the phrase path is named, exit stays zero.
    let out = ok(&wt, &["world", "ls", "--covers", "Island"]);
    assert!(out.contains("archi search"), "{out}");
    assert!(out.contains("--kind world"), "{out}");

    // A non-empty answer carries no such line.
    let out = ok(&wt, &["world", "ls", "--covers", "AuthService"]);
    assert!(!out.contains("archi search"), "{out}");

    // In JSON the note is a field of the envelope, never inside the facts.
    let v = parsed(&ok(&wt, &["world", "ls", "--covers", "Island", "--json"]));
    assert!(
        v["note"].as_str().expect("the note").contains("archi search"),
        "{v}"
    );
    assert!(v["facts"].as_array().unwrap().is_empty(), "{v}");
    let v = parsed(&ok(&wt, &["world", "ls", "--covers", "AuthService", "--json"]));
    assert!(v["note"].is_null(), "{v}");

    // The phrase path, empty, names the covers path.
    let out = ok(&wt, &["search", "dugong", "--kind", "world"]);
    assert!(out.contains("archi world ls --covers"), "{out}");
    let v = parsed(&ok(&wt, &["search", "dugong", "--kind", "world", "--json"]));
    assert!(
        v["note"]
            .as_str()
            .expect("the note")
            .contains("archi world ls --covers"),
        "{v}"
    );
    assert!(v["hits"].as_array().unwrap().is_empty(), "{v}");

    // A non-empty search carries no such line.
    let out = ok(&wt, &["search", "carriage", "--kind", "world"]);
    assert!(out.contains("trains-lose-the-signal"), "{out}");
    assert!(!out.contains("archi world ls --covers"), "{out}");
    let v = parsed(&ok(&wt, &["search", "carriage", "--kind", "world", "--json"]));
    assert!(v["note"].is_null(), "{v}");
}

/// The check prints what the wing's pass computed: its advisory lines and
/// its closing count (`the-check-counts-the-wing`).
#[test]
fn the_check_prints_the_wings_report() {
    let (_primary, wt) = temp_project();
    three_facts(&wt);

    let (_code, out, _err) = run(&wt, &["check"]);
    assert!(out.contains("world — 3 facts · 1 ungrounded"), "{out}");
    assert!(out.contains("world_unreached: Island"), "{out}");
    assert!(out.contains("world fact `tunnels-run-long`"), "{out}");

    let (_code, out, _err) = run(&wt, &["check", "--json"]);
    let v = parsed(&out);
    assert_eq!(v["world"]["facts"], 3);
    assert_eq!(v["world"]["ungrounded"], 1);
    let kinds: Vec<&str> = v["findings"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|f| f["kind"].as_str())
        .collect();
    assert!(kinds.contains(&"world_unreached"), "{out}");
    assert!(kinds.contains(&"world_state"), "{out}");

    // A tree with no wing says nothing new — the count is not born.
    fs::remove_dir_all(wt.join("archi/world")).unwrap();
    let (_code, out, _err) = run(&wt, &["check"]);
    assert!(!out.contains("world —"), "{out}");
    let (_code, out, _err) = run(&wt, &["check", "--json"]);
    assert!(parsed(&out)["world"].is_null(), "{out}");
}
