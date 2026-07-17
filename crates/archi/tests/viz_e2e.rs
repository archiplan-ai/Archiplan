//! End to end through the real binary: a subgraph query piped into `archi
//! viz` becomes an ASCII diagram. The visualizer never touches the model — it
//! reads the graph on stdin — so the model here is only what `query` needs to
//! produce one. Covers the readable diagram, the refusal of a slice too large
//! to draw, malformed input, and the `--details` listing.

use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

use serde_json::{Value, json};

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
        "archi-viz-e2e-{}-{}",
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

#[test]
fn a_query_pipes_into_a_diagram() {
    let root = temp_project();
    // The literal pipeline: `archi query --top | archi viz`.
    let (qcode, graph, _) = run(&root, &["query", "--top"], None);
    assert_eq!(qcode, 0, "query succeeds");
    let (code, out, err) = run(&root, &["viz"], Some(&graph));

    assert_eq!(code, 0, "viz succeeds\nstderr:\n{err}");
    assert!(out.contains("Orders"), "draws Orders:\n{out}");
    assert!(out.contains("Billing"), "draws Billing:\n{out}");
    assert!(out.contains("subgraph ·"), "has a caption:\n{out}");
    // OrderId is carried on the order_wire edge and is in the slice, so the
    // edge is drawn through it — Orders → OrderId → Billing — in a rounded
    // box, and the note names it as data. Only the truly edgeless nodes (the
    // preset ontology types) stay in the unconnected footnote.
    assert!(out.contains("(OrderId)"), "rounds the data box:\n{out}");
    assert!(out.contains("data carried on edges: OrderId"), "notes the data:\n{out}");
    let footnote = out.lines().find(|l| l.starts_with("unconnected in this slice:"));
    assert!(
        footnote.is_some_and(|l| !l.contains("OrderId")),
        "carried data is drawn, not footnoted:\n{out}"
    );
    // A structural diagram, never the layout engine's cycle bail-out.
    assert!(!out.contains("CYCLE DETECTED"), "{out}");
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn refuses_a_slice_too_large_to_draw() {
    let root = temp_project();
    let nodes: Vec<Value> = (0..25).map(|i| json!({ "id": format!("N{i}") })).collect();
    let big = json!({ "nodes": nodes, "edges": [] }).to_string();

    let (code, out, err) = run(&root, &["viz"], Some(&big));
    assert_eq!(code, 1, "refusal is a distinct non-zero exit");
    assert!(out.is_empty(), "nothing is drawn:\n{out}");
    assert!(err.contains("too large to visualize"), "{err}");
    assert!(err.contains("25 nodes"), "{err}");
    // The refusal is actionable: it names the narrowing commands.
    assert!(err.contains("archi query"), "{err}");
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn rejects_input_that_is_not_a_graph() {
    let root = temp_project();
    let (code, _, err) = run(&root, &["viz"], Some("{\"status\":\"ok\"}"));
    assert_eq!(code, 2, "malformed input is a usage-class error");
    assert!(err.contains("archi viz"), "guidance names the tool:\n{err}");
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn details_flag_reveals_the_collapsed_attributes() {
    let root = temp_project();
    let (_, graph, _) = run(&root, &["query", "--top"], None);

    // Without --details the diagram stays structural.
    let (_, plain, _) = run(&root, &["viz"], Some(&graph));
    assert!(!plain.contains("details"), "withheld by default:\n{plain}");

    // With it, the carried node the diagram omits is listed.
    let (code, out, _) = run(&root, &["viz", "--details"], Some(&graph));
    assert_eq!(code, 0);
    assert!(out.contains("details"), "{out}");
    assert!(out.contains("carries OrderId"), "{out}");
    fs::remove_dir_all(&root).unwrap();
}
