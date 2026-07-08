//! End to end through the real binary: the agent read envelope
//! (`requirements/agent-interface.md`) — a batch of reads in, the response
//! envelope out; writes are protocol errors; `--at` reads a version
//! reconstructed from the sealed archive.

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
    dir
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
