//! End to end through the real binary: the agent read envelope
//! (`archi/requirements/self-hosting/agents-read-lowered-statements.md`) — a batch of reads in, the response
//! envelope out; writes are protocol errors; `--at` reads a version
//! reconstructed from the sealed archive. The answer also carries what
//! conditions it: the world facts covering the elements the slice names
//! (`archi/requirements/world-facts/the-read-envelope-carries-the-conditions.md`).

mod util;

use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

use serde_json::Value;

static NEXT: AtomicUsize = AtomicUsize::new(0);

const MODEL: &str = "def conn wire := * -> *\n\
                     def node OrderId\n\
                     def conn order_wire := * ->OrderId *\n\
                     def node Orders:\n  port events\n  port confirms\n\
                     def node Billing:\n  port inn\n  port book\n\
                     Orders.events wire Billing.inn\n\
                     Orders.confirms order_wire Billing.book\n";

fn temp_project() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "archi-read-e2e-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::SeqCst)
    ));
    fs::create_dir_all(dir.join("archi/src")).unwrap();
    fs::write(dir.join("archi.toml"), "[project]\nname = \"t\"\n").unwrap();
    fs::write(dir.join("archi/src/model.arch"), MODEL).unwrap();
    util::worktree(&dir)
}

/// Run the binary; return (exit code, stdout, stderr).
fn run(root: &Path, args: &[&str], stdin: Option<&str>) -> (i32, String, String) {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_archi"));
    cmd.args(args)
        .args(["--project", root.to_str().unwrap()])
        .stdin(if stdin.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = cmd.spawn().expect("archi spawns");
    if let Some(text) = stdin {
        child
            .stdin
            .take()
            .expect("stdin is piped")
            .write_all(text.as_bytes())
            .unwrap();
    }
    let out = child.wait_with_output().expect("archi finishes");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

fn json(text: &str) -> Value {
    serde_json::from_str(text).unwrap_or_else(|e| panic!("not JSON ({e}):\n{text}"))
}

fn node_ids(result: &Value) -> Vec<String> {
    result["nodes"]
        .as_array()
        .expect("a graph has nodes")
        .iter()
        .map(|n| n["id"].as_str().unwrap().to_string())
        .collect()
}

/// The strings of a JSON array of strings.
fn strings(value: &Value) -> Vec<&str> {
    value
        .as_array()
        .unwrap_or_else(|| panic!("an array of strings, not {value}"))
        .iter()
        .map(|v| v.as_str().expect("a string"))
        .collect()
}

/// The second half of the world fixture's model: a datum of its own, so the
/// carrier filter can compose a slice of elements no fact covers, and an
/// island no such slice ever names.
const WORLD_MODEL: &str = "\
def node Ledger:
  port post
def node Vault:
  port keep
def node LedgerId
def node Island
def conn ledger_wire := * ->LedgerId *
Ledger.post ledger_wire Vault.keep
";

/// One whole world fact under `archi/world/facts/`: the covers list as given,
/// the conditioning paragraph, the workaround and one scenario named after the
/// fact.
fn fact(root: &Path, slug: &str, title: &str, covers: &str, condition: &str) {
    util::Fact {
        covers,
        sources: "https://example.org/thread/42",
        uses: "",
        condition,
        workaround: "Riders load the page at the platform and redo the trip's work when they \
                     forget.",
        scenarios: &format!(
            "### {slug} holds\n\n\
             Given the device has no network\nWhen the user opens the app\n\
             Then the last synced view appears\n"
        ),
    }
    .write(root, slug, title);
}

/// A project on both halves of the model, with the world when `world` says so:
/// one fact on a node the carrier slice names, one on two of its nodes at
/// once, one on a port it names, and one on the island it never reaches.
fn world_project(world: bool) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "archi-read-world-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::SeqCst)
    ));
    fs::create_dir_all(dir.join("archi/src")).unwrap();
    fs::write(dir.join("archi.toml"), "[project]\nname = \"t\"\n").unwrap();
    fs::write(
        dir.join("archi/src/model.arch"),
        format!("{MODEL}{WORLD_MODEL}"),
    )
    .unwrap();
    if world {
        fact(
            &dir,
            "riders-lose-the-signal",
            "Riders lose the signal",
            "Orders",
            "The carriage drops the network for minutes at a time.",
        );
        fact(
            &dir,
            "tunnels-run-long",
            "Tunnels run long",
            "Orders, Billing",
            "A tunnel runs for minutes on the northern line.",
        );
        fact(
            &dir,
            "the-books-close-at-night",
            "The books close at night",
            "Billing.book",
            "The clearing house stops taking entries at midnight.",
        );
        fact(
            &dir,
            "the-guard-walks-the-line",
            "The guard walks the line",
            "Island",
            "A guard walks the length of the platform every hour.",
        );
    }
    dir
}

/// The answer carries what conditions it
/// (`the-read-envelope-carries-the-conditions`): the facts covering the
/// elements the composed slice names, each once, under their own key.
#[test]
fn the_read_envelope_carries_the_conditions() {
    let root = world_project(true);

    // The slice names `Orders`, `Billing` and the port `Billing.book`; the
    // facts on those ride with it, in slug order. The fact on the island the
    // slice never names stays behind, and so does the port it never draws.
    let (code, out, _) = run(&root, &["query", "--carrier", "OrderId"], None);
    assert_eq!(code, 0, "{out}");
    let v = json(&out);
    let facts = v["world"].as_array().expect("the covering facts");
    let slugs: Vec<&str> = facts.iter().map(|f| f["slug"].as_str().unwrap()).collect();
    assert_eq!(
        slugs,
        [
            "riders-lose-the-signal",
            "the-books-close-at-night",
            "tunnels-run-long"
        ],
        "{out}"
    );

    // A fact covering two elements of one slice appears once.
    assert_eq!(
        slugs.iter().filter(|s| **s == "tunnels-run-long").count(),
        1,
        "{out}"
    );

    // Each fact carries its address, what it conditions, the statement and
    // the scenarios it dictates.
    let f = &facts[0];
    assert_eq!(f["path"], "archi/world/facts/riders-lose-the-signal.md");
    assert_eq!(strings(&f["covers"]), ["Orders"]);
    assert_eq!(
        f["condition"],
        "The carriage drops the network for minutes at a time."
    );
    assert_eq!(f["scenarios"][0]["name"], "riders-lose-the-signal holds");
    assert_eq!(
        strings(&f["scenarios"][0]["steps"]),
        [
            "Given the device has no network",
            "When the user opens the app",
            "Then the last synced view appears"
        ]
    );
    assert_eq!(strings(&facts[2]["covers"]), ["Orders", "Billing"]);

    // The envelope carries them under the same key, beside the results and
    // never inside the nodes.
    let (code, out, _) = run(
        &root,
        &["read", "-"],
        Some(r#"{"statements":[{"stmt":"query","carriers":["OrderId"]},{"stmt":"check"}]}"#),
    );
    assert_eq!(code, 0, "{out}");
    let v = json(&out);
    assert_eq!(v["status"], "ok");
    assert_eq!(v["results"][0]["result"], "graph");
    assert_eq!(v["results"][1]["result"], "findings");
    let slugs: Vec<&str> = v["world"]
        .as_array()
        .expect("the covering facts")
        .iter()
        .map(|f| f["slug"].as_str().unwrap())
        .collect();
    assert_eq!(
        slugs,
        [
            "riders-lose-the-signal",
            "the-books-close-at-night",
            "tunnels-run-long"
        ],
        "{out}"
    );
    for node in v["results"][0]["nodes"].as_array().unwrap() {
        assert!(node["world"].is_null(), "the facts ride the envelope: {out}");
    }

    fs::remove_dir_all(&root).unwrap();
}

/// A slice no fact covers carries the slice alone, and a tree with no world
/// answers exactly as it did before the world existed — byte for byte, from
/// `query` and from `read` both.
#[test]
fn a_slice_no_fact_covers_carries_the_slice_alone() {
    let with_world = world_project(true);
    let bare = world_project(false);

    // `Ledger`, `Vault` and `LedgerId`: three elements, no fact on any of
    // them. The world writes no key, and no byte separates the two trees.
    let (code, world_out, _) = run(&with_world, &["query", "--carrier", "LedgerId"], None);
    assert_eq!(code, 0, "{world_out}");
    assert_eq!(
        node_ids(&json(&world_out)),
        ["Ledger", "LedgerId", "Vault"],
        "{world_out}"
    );
    assert!(json(&world_out)["world"].is_null(), "{world_out}");
    let (_, bare_out, _) = run(&bare, &["query", "--carrier", "LedgerId"], None);
    assert_eq!(world_out, bare_out);

    // The read envelope over the same slice, byte for byte.
    let request = r#"{"statements":[{"stmt":"query","carriers":["LedgerId"]},{"stmt":"check"}]}"#;
    let (code, world_out, _) = run(&with_world, &["read", "-"], Some(request));
    assert_eq!(code, 0, "{world_out}");
    let (_, bare_out, _) = run(&bare, &["read", "-"], Some(request));
    assert_eq!(world_out, bare_out);

    // A tree with no `archi/world/` says nothing new about any slice.
    let (code, out, _) = run(&bare, &["query", "--carrier", "OrderId"], None);
    assert_eq!(code, 0, "{out}");
    assert!(json(&out)["world"].is_null(), "{out}");

    fs::remove_dir_all(&with_world).unwrap();
    fs::remove_dir_all(&bare).unwrap();
}

#[test]
fn the_envelope_reads_batches_and_refuses_writes() {
    let root = temp_project();

    // A batch of reads: one result per statement, in order, verbatim shape.
    let request = r#"{"statements":[
        {"stmt":"query","scopes":["Orders"],"kinds":["connection"]},
        {"stmt":"check"}
    ]}"#;
    let (code, out, _) = run(&root, &["read", "-"], Some(request));
    assert_eq!(code, 0, "{out}");
    let v = json(&out);
    assert_eq!(v["status"], "ok");
    assert_eq!(v["results"][0]["result"], "graph");
    assert!(node_ids(&v["results"][0]).contains(&"Orders".to_string()));
    assert_eq!(v["results"][1]["result"], "findings");

    // Piped stdin with no positional reads the same way; a file does too.
    let (code, _, _) = run(&root, &["read"], Some(r#"{"statements":[]}"#));
    assert_eq!(code, 0);
    let req_file = root.join("req.json");
    fs::write(&req_file, r#"{"statements":[{"stmt":"check"}]}"#).unwrap();
    let (code, out, _) = run(&root, &["read", req_file.to_str().unwrap()], None);
    assert_eq!(code, 0, "{out}");

    // A write statement is a protocol error: E_BAD_REQUEST, no index,
    // exit 2 — the model is edited as source, never through the envelope.
    let (code, out, _) = run(
        &root,
        &["read", "-"],
        Some(r#"{"statements":[{"stmt":"define","node":"Rogue"}]}"#),
    );
    assert_eq!(code, 2, "{out}");
    let v = json(&out);
    assert_eq!(v["status"], "error");
    assert_eq!(v["error"]["code"], "E_BAD_REQUEST");
    assert!(v["error"]["index"].is_null());

    // A failing read carries the statement's index; exit 1.
    let (code, out, _) = run(
        &root,
        &["read", "-"],
        Some(r#"{"statements":[{"stmt":"query","views":["ghost"]}]}"#),
    );
    assert_eq!(code, 1, "{out}");
    let v = json(&out);
    assert_eq!(v["error"]["index"], 0);
    assert_eq!(v["error"]["code"], "E_UNKNOWN_NAME");

    // Invalid JSON is the same protocol-error envelope, before the engine.
    let (code, out, _) = run(&root, &["read", "-"], Some("not json"));
    assert_eq!(code, 2, "{out}");
    assert_eq!(json(&out)["error"]["code"], "E_BAD_REQUEST");

    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn query_composes_filters_and_reads_sealed_versions() {
    let root = temp_project();

    // Unfiltered: the whole graph, one connection edge.
    let (code, out, _) = run(&root, &["query"], None);
    assert_eq!(code, 0, "{out}");
    let v = json(&out);
    assert_eq!(v["result"], "graph");
    let ids = node_ids(&v);
    assert!(ids.contains(&"Orders".to_string()) && ids.contains(&"Billing".to_string()));
    let mut edge_types: Vec<String> = v["edges"]
        .as_array()
        .unwrap()
        .iter()
        .inspect(|e| assert_eq!(e["kind"], "connection"))
        .map(|e| e["type"].as_str().unwrap().to_string())
        .collect();
    edge_types.sort();
    assert_eq!(edge_types, vec!["order_wire".to_string(), "wire".to_string()]);

    // A kind filter restricts edges; unknown names error humanly, exit 1.
    let (code, out, _) = run(&root, &["query", "--kind", "relation"], None);
    assert_eq!(code, 0);
    assert!(json(&out)["edges"].as_array().unwrap().is_empty());
    let (code, _, err) = run(&root, &["query", "--view", "ghost"], None);
    assert_eq!(code, 1);
    assert!(err.contains("unknown view"), "{err}");

    // A carrier filter slices the flow of a datum — the carrying edges plus
    // only the nodes related to them; an edge-type filter slices by name.
    let (code, out, _) = run(&root, &["query", "--carrier", "OrderId"], None);
    assert_eq!(code, 0, "{out}");
    let v = json(&out);
    assert_eq!(v["edges"].as_array().unwrap().len(), 1);
    assert_eq!(v["edges"][0]["type"], "order_wire");
    let mut related = node_ids(&v);
    related.sort();
    assert_eq!(
        related,
        vec!["Billing".to_string(), "OrderId".to_string(), "Orders".to_string()]
    );
    let (code, out, _) = run(&root, &["query", "--edge-type", "wire"], None);
    assert_eq!(code, 0, "{out}");
    let v = json(&out);
    assert_eq!(v["edges"].as_array().unwrap().len(), 1);
    assert_eq!(v["edges"][0]["type"], "wire");
    let (code, _, err) = run(&root, &["query", "--edge-type", "ghost"], None);
    assert_eq!(code, 1);
    assert!(err.contains("edge-type"), "{err}");

    // Seal a version, grow the model: Working sees the growth, `--at` the
    // pin — how an agent grounds itself against a plan's pinned spec.
    let (code, _, _) = run(&root, &["version", "save", "-m", "first"], None);
    assert_eq!(code, 0);
    fs::write(
        root.join("archi/src/model.arch"),
        format!("{MODEL}def node Ledger\n"),
    )
    .unwrap();
    let (_, out, _) = run(&root, &["query"], None);
    assert!(node_ids(&json(&out)).contains(&"Ledger".to_string()));
    let (code, out, _) = run(&root, &["query", "--at", "v0001"], None);
    assert_eq!(code, 0, "{out}");
    assert!(!node_ids(&json(&out)).contains(&"Ledger".to_string()));
    let (code, _, err) = run(&root, &["query", "--at", "v9999"], None);
    assert_eq!(code, 1);
    assert!(err.contains("v9999"), "{err}");

    // `read` takes --at the same way.
    let (code, out, _) = run(
        &root,
        &["read", "-", "--at", "v0001"],
        Some(r#"{"statements":[{"stmt":"query"}]}"#),
    );
    assert_eq!(code, 0, "{out}");
    assert!(!node_ids(&json(&out)["results"][0]).contains(&"Ledger".to_string()));

    fs::remove_dir_all(&root).unwrap();
}
