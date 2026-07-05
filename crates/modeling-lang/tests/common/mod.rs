#![allow(dead_code)]

use modeling_lang::{
    ErrorCode, Finding, GraphEdge, GraphNode, LangError, Outcome, Statement, Workspace,
};
use serde_json::Value;

pub fn ws_with(batch: Value) -> Workspace {
    let mut ws = Workspace::new();
    match ws.execute_values(batch.as_array().expect("batch is an array")) {
        Ok(_) => ws,
        Err(e) => panic!("setup batch failed: {e}"),
    }
}

pub fn outcomes(ws: &mut Workspace, batch: Value) -> Vec<Outcome> {
    ws.execute_values(batch.as_array().expect("batch is an array"))
        .unwrap_or_else(|e| panic!("batch failed: {e}"))
}

pub fn outcome(ws: &mut Workspace, stmt: Value) -> Outcome {
    outcomes(ws, Value::Array(vec![stmt]))
        .pop()
        .expect("one outcome")
}

pub fn err(ws: &mut Workspace, batch: Value) -> (usize, LangError) {
    match ws.execute_values(batch.as_array().expect("batch is an array")) {
        Ok(r) => panic!("expected failure, got {r:?}"),
        Err(b) => (b.index, b.error),
    }
}

pub fn err_code(ws: &mut Workspace, stmt: Value) -> ErrorCode {
    err(ws, Value::Array(vec![stmt])).1.code
}

pub fn graph(ws: &mut Workspace, stmt: Value) -> (Vec<GraphNode>, Vec<GraphEdge>) {
    match outcome(ws, stmt) {
        Outcome::Graph { nodes, edges } => (nodes, edges),
        o => panic!("expected a graph, got {o:?}"),
    }
}

/// Node ids of a query result, in result (creation) order.
pub fn node_ids(ws: &mut Workspace, stmt: Value) -> Vec<String> {
    graph(ws, stmt).0.into_iter().map(|n| n.id).collect()
}

/// Edges of a query result as JSON values, for shape-exact assertions.
pub fn edge_values(ws: &mut Workspace, stmt: Value) -> Vec<Value> {
    graph(ws, stmt)
        .1
        .iter()
        .map(|e| serde_json::to_value(e).expect("edges serialize"))
        .collect()
}

/// The model rendered as pseudo-syntax lines, for state assertions.
pub fn dump_pseudo(ws: &Workspace) -> Vec<String> {
    ws.model().dump().iter().map(Statement::pseudo).collect()
}

pub fn findings(ws: &mut Workspace, stmt: Value) -> Vec<Finding> {
    match outcome(ws, stmt) {
        Outcome::Findings { findings } => findings,
        o => panic!("expected findings, got {o:?}"),
    }
}

pub fn cascade(ws: &mut Workspace, stmt: Value) -> Vec<String> {
    match outcome(ws, stmt) {
        Outcome::Applied { cascade: Some(c) } => c.iter().map(Statement::pseudo).collect(),
        o => panic!("expected a cascade, got {o:?}"),
    }
}

pub fn is_applied(o: &Outcome) -> bool {
    matches!(o, Outcome::Applied { .. })
}

pub fn is_noop(o: &Outcome) -> bool {
    matches!(o, Outcome::Noop)
}
