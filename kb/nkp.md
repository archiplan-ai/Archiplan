# Feature Spec: NKP Analysis
**System**: Architecture Specification Engine  
**Representation**: Typed-node / labeled-edge graph (two-primitive ontology)  
**Version**: 1.0  
**Status**: Draft

---

## 1. Purpose

Extend the architecture spec system with NKP landscape analysis. Given an architecture graph, the system shall compute the **ruggedness** of its design space, visualize component coupling structure, and identify **neutral corridors** — regions of the graph where structural changes preserve global fitness, i.e. safe refactoring zones.

The analysis maps the architecture graph to a Kauffman NK fitness landscape, augmented with the P neutrality parameter, and produces three primary artifacts:

1. **Landscape metrics** — K̄, P̄, local optima estimate, regime classification
2. **Dependency matrix** — visual N×N coupling heatmap with cluster decomposition
3. **Refactoring safety report** — neutral corridor set, high-risk node set, decoupling candidates

---

## 2. Definitions

| Symbol | Meaning |
|--------|---------|
| G = (V, E) | Architecture graph; V = typed nodes, E = labeled directed edges |
| N | \|V\| — number of component nodes in scope |
| K_i | In-degree of node i restricted to **epistatic** edges in scope |
| K̄ | Mean K_i over all i ∈ V |
| f_i(s_i, s_{N_i}) | Fitness contribution of node i given its state and states of its K_i neighbors N_i |
| F(s) | Global fitness = (1/N) · Σ f_i — normalized sum of contributions |
| P_i | Probability that a fitness contribution of node i is neutral (zero) |
| P̄ | Mean P_i over all i ∈ V |
| Neutral corridor | Maximal connected subgraph where all nodes have P_i above threshold τ_P |
| Local optimum | State s* such that flipping any single node's state does not increase F(s*) |

### 2.5 Layer Ontology

The architecture graph is partitioned into two **layers**, declared on each node type and edge type:

- **Epistatic** — runtime/structural coupling: components, services, data paths, and the edges that carry fitness interactions. NKP (K̄, P̄, regime, corridors, hotspots, dependency matrix) applies **only** to this layer.
- **Epistemic** — ontology, metadata, requirements, stressors, and other “about the spec” concepts. These nodes and edges do not participate in the NK fitness machinery; analyzing them with regime metrics is a category error.

**Edge-type rule (cross-layer invariant):** for an edge type whose layer is **Epistatic**, every `from_type` and `to_type` in its catalog must be **Epistatic**. **Epistemic** edge types may reference any node types. Epistemic content may link *to* epistatic nodes (e.g. requirements targeting a service), but epistatic edge types cannot bridge into epistemic-only node types.

**Property scope:** fitness contributions, regime classification, neutral corridors, coupling hotspots, and the N×N dependency matrix are **properties of the epistatic layer only**. They are unchanged by adding or removing epistemic nodes and edges (see acceptance criteria G3–G5).

---

## 3. Input Contract

### 3.1 Graph Preconditions

The NKP analyzer operates with **`layer = Epistatic`** by default. It accepts any subgraph of the architecture graph satisfying:

- Every node in scope has an **Epistatic** node type (e.g. Component and its subtypes in a typical catalog).
- Every edge counted toward K_i is an **Epistatic** edge type whose endpoints are both Epistatic nodes (enforced at type-definition time for epistatic edges).
- Optionally, **`only_edge_types`** restricts which epistatic edge type names contribute; if omitted, all epistatic edge types in the graph are included.
- Epistemic nodes and edges are **not** part of the K-matrix or NKP metrics under default parameters.
- The graph may be disconnected; analysis runs per weakly-connected component, then aggregates.

### 3.2 Fitness Assignment

Fitness contributions are assigned via one of three strategies, selectable by the user:

| Strategy | Description |
|----------|-------------|
| `UNIFORM_RANDOM` | f_i drawn i.i.d. from Uniform[0,1] per (state, neighbor-state) tuple. Used for baseline theoretical analysis. |
| `WEIGHT_LABELED` | f_i derived from edge weight annotations on epistatic edges. Requires edges to carry a `weight: Float ∈ [0,1]` attribute. |
| `STABILITY_PROXY` | f_i computed from node stability score: inverse of out-degree normalized by N. High out-degree = high epistatic influence = lower fitness contribution per state. |

Default strategy: `STABILITY_PROXY` (most meaningful for architecture graphs without explicit weights).

### 3.3 Neutrality Assignment

P_i is assigned via one of two strategies:

| Strategy | Description |
|----------|-------------|
| `UNIFORM_P` | Single global P value, user-supplied ∈ [0,1]. All fitness tables have P fraction of entries zeroed. |
| `DEGREE_DERIVED` | P_i = 1 − (K_i / K_max). Nodes with low in-degree have high neutrality (few dependencies → more configurations are fitness-equivalent). |

Default strategy: `DEGREE_DERIVED`.

---

## 4. Analysis Pipeline

```
Architecture Graph G
        │
        ▼
┌───────────────────┐
│  Layer Filter     │  Epistatic-layer nodes + epistatic edges (optional only_edge_types)
└────────┬──────────┘
         │
         ▼
┌───────────────────┐
│  K-Matrix Build   │  Compute N×N adjacency on filtered epistatic edges
└────────┬──────────┘
         │
         ▼
┌───────────────────┐
│  Fitness Tables   │  Assign f_i per chosen strategy
└────────┬──────────┘
         │
         ▼
┌───────────────────┐
│  P-Mask Apply     │  Zero out neutrality fraction of each f_i table
└────────┬──────────┘
         │
         ├──────────────────────┬────────────────────────┐
         ▼                      ▼                        ▼
┌─────────────────┐   ┌──────────────────┐   ┌──────────────────────┐
│ Metric Compute  │   │ Matrix Render    │   │ Neutral Corridor Det │
└────────┬────────┘   └────────┬─────────┘   └──────────┬───────────┘
         │                     │                         │
         └─────────────────────┴─────────────────────────┘
                                │
                                ▼
                        NKP Analysis Report
```

### 4.1 Metric Computation

**K̄ (mean connectivity)**:
```
K̄ = (1/N) · Σ_i K_i
```

**P̄ (mean neutrality)**:
```
P̄ = (1/N) · Σ_i P_i
```

**Regime classification** (deterministic, based on K̄):

| K̄ range | Regime | Label |
|----------|--------|-------|
| K̄ < 1.0 | Ordered | `ORDERED` |
| 1.0 ≤ K̄ ≤ 3.0 | Edge of chaos | `CRITICAL` |
| K̄ > 3.0 | Chaotic | `CHAOTIC` |

**Local optima estimate** (analytical approximation):
```
Ω_est = 2^N / (K̄ + 1)
```
For large N this is reported as log₂(Ω_est) to avoid overflow.

**Adaptive walk length** (empirical, via simulation):  
Run M = min(100, 2^N) hill-climbing walks from random start states. At each step, evaluate all N single-node flips; accept the best improvement; terminate when no improvement exists. Record convergence step count. Report mean and std.

**Correlation length** ξ:  
```
ξ ≈ 1 / K̄   (Weinberger approximation)
```
Interpretive note: ξ < 1 means changes are uncorrelated beyond nearest neighbors.

### 4.2 Dependency Matrix

Output: N×N matrix M where:
```
M[i][j] = 1  iff  edge (j → i) exists with an epistatic edge type in scope
M[i][j] = 0  otherwise
```

Annotation layers rendered on top of the raw matrix:

- **Cluster decomposition**: Apply spectral clustering (k chosen via eigengap heuristic on graph Laplacian, k_max = 8). Color-code clusters. Clusters with high intra-cluster density and low inter-cluster density are **modular subsystems**.
- **Hotspot overlay**: Nodes with K_i > K̄ + σ_K are marked as **coupling hotspots** (red). These are the highest-risk refactoring targets.
- **P overlay**: Cell opacity encodes P_i of the row node. High-opacity rows = high neutrality = safe to restructure.

### 4.3 Neutral Corridor Detection

Algorithm:

```
1. Compute P_i for all nodes under chosen strategy
2. Mark node i as NEUTRAL if P_i ≥ τ_P  (default τ_P = 0.6)
3. Build induced subgraph G_N of all NEUTRAL nodes
4. Extract weakly-connected components of G_N
5. For each component C:
   a. Compute internal K̄_C (mean in-degree within C)
   b. Compute boundary degree: number of edges from C to non-neutral nodes
   c. Label C as SAFE_CORRIDOR if boundary_degree / |C| < τ_B  (default τ_B = 0.3)
      else label as PARTIALLY_NEUTRAL
6. Report SAFE_CORRIDOR components as refactoring zones
```

### 4.4 Epistemic Layer Health

The epistemic layer is not an NK landscape. For coverage of requirements, subtype discipline, and stressor/requirement hygiene, use **`fractal query ontology`** (and subcommands). The implementation reports four metrics (see layering task 13):

1. **Coverage** — fraction of epistatic nodes that have at least one incoming edge of the configured epistemic link types (default: all epistemic edge types).
2. **Consistency** — orphan epistemic nodes, dangling `subtype_of` chains, and subtype cycles.
3. **Provenance** — freestanding requirements, unresolved stressors, dangling `derive_req` links.
4. **Density** — requirement load per epistatic node and a simple histogram (min / p50 / max).

Use the full report via `fractal query ontology report`, or invoke `coverage`, `consistency`, `provenance`, and `density` individually.

Output per corridor:
- Node set (ids + names)
- Internal K̄_C
- Boundary exposure ratio
- Suggested action: `ENCAPSULATE` | `EXTRACT_MODULE` | `SIMPLIFY_INTERFACE`

Action rules:
- `ENCAPSULATE` — corridor has low boundary exposure and low internal K̄_C (boundary < τ_B AND K̄_C < 1)
- `EXTRACT_MODULE` — corridor is large (\|C\| > N/4) and internally cohesive
- `SIMPLIFY_INTERFACE` — corridor has moderate boundary exposure (τ_B ≤ boundary < 2·τ_B)

---

## 5. Output Schema

```typescript
interface NKPReport {
  scope: {
    node_count: number;           // N
    edge_count: number;           // epistatic edges in scope only
    fitness_strategy: FitnessStrategy;
    neutrality_strategy: NeutralityStrategy;
  };

  metrics: {
    K_bar: number;                // mean connectivity
    K_std: number;                // std dev of K_i
    P_bar: number;                // mean neutrality
    regime: "ORDERED" | "CRITICAL" | "CHAOTIC";
    local_optima_log2: number;    // log₂(Ω_est)
    correlation_length: number;   // ξ
    adaptive_walk: {
      mean_steps: number;
      std_steps: number;
      sample_count: number;
    };
  };

  dependency_matrix: {
    nodes: NodeRef[];             // ordered list, defines row/col index
    matrix: number[][];           // N×N binary adjacency
    clusters: Cluster[];          // spectral decomposition result
    hotspots: NodeRef[];          // nodes where K_i > K̄ + σ_K
  };

  neutral_corridors: NeutralCorridor[];

  warnings: Warning[];           // e.g. N too small for reliable stats, disconnected graph
}

interface NeutralCorridor {
  id: string;
  nodes: NodeRef[];
  K_bar_internal: number;
  boundary_exposure: number;
  action: "ENCAPSULATE" | "EXTRACT_MODULE" | "SIMPLIFY_INTERFACE";
  confidence: number;            // heuristic: 1 − boundary_exposure
}

interface Warning {
  code: "SMALL_N" | "DISCONNECTED" | "NO_NEUTRAL_NODES" | "FULLY_CHAOTIC";
  message: string;
}
```

---

## 6. Parameterization

All thresholds are user-overridable at invocation time:

| Parameter | Default | Description |
|-----------|---------|-------------|
| `fitness_strategy` | `STABILITY_PROXY` | Fitness assignment method |
| `neutrality_strategy` | `DEGREE_DERIVED` | P assignment method |
| `global_P` | `0.5` | Used only when `neutrality_strategy = UNIFORM_P` |
| `tau_P` | `0.6` | Neutrality threshold for corridor detection |
| `tau_B` | `0.3` | Boundary exposure threshold for SAFE_CORRIDOR |
| `walk_samples` | `100` | Adaptive walk simulation count |
| `cluster_k_max` | `8` | Max clusters in spectral decomposition |
| `layer` | `Epistatic` | Which layer’s node/edge types participate in NKP; epistemic layer is not a fitness landscape |
| `only_edge_types` | none (all epistatic edge types) | Optional list of epistatic edge type names to include in K and metrics; omit to use every epistatic edge type |

---

## 7. Integration Points

### 7.1 Graph Query Interface

The analyzer requires the host system to expose:

```typescript
interface GraphQueryAdapter {
  // Return all nodes whose type is in the Epistatic layer (and optional scope)
  getEpistaticNodes(scope?: ScopeFilter): Node[];

  // Return epistatic edges between nodes, optionally filtered by only_edge_types
  getEpistaticEdges(nodes: Node[], onlyEdgeTypes?: string[]): Edge[];

  // Return node metadata (name, type, stability_score if available)
  getNodeMeta(id: NodeId): NodeMeta;
}
```

### 7.2 Invocation

```typescript
const report: NKPReport = await analyzeNKP(graphAdapter, {
  fitness_strategy: "STABILITY_PROXY",
  neutrality_strategy: "DEGREE_DERIVED",
  tau_P: 0.6,
  tau_B: 0.3,
});
```

### 7.3 Rendering

The report object is self-contained and renderable independently of the graph store. The dependency matrix is dense (N×N numbers) — for N > 200, the renderer should apply row/column reordering by cluster index before display to make block structure visible.

---

## 8. Edge Cases and Constraints

| Condition | Handling |
|-----------|----------|
| N < 4 | Emit `SMALL_N` warning; metrics computed but unreliable |
| N > 500 | Skip adaptive walk simulation; emit note in report |
| K̄ = 0 (no epistatic edges in scope) | Emit `ORDERED` regime; all nodes are safe corridors |
| Fully disconnected graph | Run per-component, then report aggregate K̄ weighted by component size |
| No nodes pass P_i ≥ τ_P | Emit `NO_NEUTRAL_NODES` warning; suggest lowering τ_P or switching neutrality strategy |
| All nodes are neutral | Entire graph is one corridor; emit as single `EXTRACT_MODULE` suggestion |

---

## 9. Non-Goals (Out of Scope for v1.0)

- Coupled NK landscapes (multi-agent / co-evolutionary analysis between subsystems)
- Continuous-valued node states (binary states only in v1.0)
- Temporal fitness landscape evolution (landscape changes as architecture evolves)
- Automatic application of refactoring suggestions (report only; no write-back to graph)
- NKQ variant (discretized fitness values)

---

## 10. Acceptance Criteria

1. Given a graph with N=10, K̄=0 (no epistatic edges in scope), report must classify regime as `ORDERED` and all nodes must appear in a single neutral corridor.
2. Given a fully-connected graph (K̄ = N−1 = 9), report must classify regime as `CHAOTIC` and neutral corridor set must be empty (or emit `NO_NEUTRAL_NODES`).
3. Given a graph with two clearly separable clusters (dense internal edges, zero inter-cluster edges), spectral decomposition must identify exactly 2 clusters.
4. Adaptive walk mean step count must be monotonically non-decreasing as K̄ increases across a test sweep K̄ ∈ {0, 1, 2, 4, 8} on N=20 random graphs (averaged over 50 runs per K̄).
5. `NKPReport` must serialize to valid JSON and satisfy the TypeScript schema without runtime errors for any valid input graph.
6. **G3** — `build_k_matrix` with default parameters (`layer = Epistatic`, no `only_edge_types` restriction) yields the same matrix for two specs that differ only in epistemic nodes/edges.
7. **G4** — Adding epistemic nodes or edges must not change K̄, P̄, regime, or `local_optima_log2` for the same epistatic subgraph as before.
8. **G5** — On an epistatic-only graph, metrics must match whether `only_edge_types` is omitted or lists every epistatic edge type name explicitly.
