# egui-nodes-app

![Rust CI](https://github.com/atomic-media-studio/egui-app/actions/workflows/rust-ci.yml/badge.svg)

Repository structure:

```text
├── Cargo.toml                 # workspace root
├── src/                       # demo binary (default `cargo run`)
│   ├── main.rs
│   └── style_panel.rs         # canvas / NodesStyle tuning for the demo
└── crates/
    ├── dag-lib/               # Rust crate `dag_lib`
    └── egui-nodes/            # Rust crate `egui_nodes`
        └── src/
            ├── lib.rs         # public re-exports (see below)
            ├── io.rs          # load_graph / save_graph
            └── ui/            # editor, nodes_engine (NodeGraph + canvas), view, style, …
```

- **`dag-lib`** (`[crates/dag-lib](crates/dag-lib)`) — Headless **directed** graph: [`Graph`](crates/dag-lib/src/model.rs), [`Node`](crates/dag-lib/src/model.rs) / [`Link`](crates/dag-lib/src/model.rs) / [`Pin`](crates/dag-lib/src/model.rs), opaque ids ([`NodeId`](crates/dag-lib/src/ids.rs), [`PinId`](crates/dag-lib/src/ids.rs), [`LinkId`](crates/dag-lib/src/ids.rs)), [`dependency_graph_is_acyclic`](crates/dag-lib/src/eval.rs), [`compute_topological_order`](crates/dag-lib/src/eval.rs), [`Executor`](crates/dag-lib/src/eval.rs) / [`NodeEvaluator`](crates/dag-lib/src/eval.rs) / [`Value`](crates/dag-lib/src/eval.rs). Optional **`serde`**. No egui.
- **`egui-nodes`** (`[crates/egui-nodes](crates/egui-nodes)`) — Use **`egui_nodes::…`**. Depends on **`dag-lib`** + **egui**. Re-exports **`dag_lib`** (also as `egui_nodes::dag_lib`), **`egui_nodes::nodes_engine`** (slab [`NodeGraph`](crates/egui-nodes/src/ui/nodes_engine/mod.rs), canvas under [`nodes_engine::canvas`](crates/egui-nodes/src/ui/nodes_engine/canvas/mod.rs)), and the UI surface in [`lib.rs`](crates/egui-nodes/src/lib.rs): [`NodesEditor`](crates/egui-nodes/src/ui/editor/mod.rs), [`NodesView`](crates/egui-nodes/src/ui/view.rs), [`NodeData`](crates/egui-nodes/src/ui/editor/mod.rs), [`GraphChanges`](crates/egui-nodes/src/ui/editor/mod.rs), [`NodesStyle`](crates/egui-nodes/src/ui/style.rs), [`layout_to_pos2`](crates/egui-nodes/src/ui/editor/mod.rs) / [`pos2_to_layout`](crates/egui-nodes/src/ui/editor/mod.rs), [`load_graph`](crates/egui-nodes/src/io.rs) / [`save_graph`](crates/egui-nodes/src/io.rs), style hooks ([`NodeStyleHook`](crates/egui-nodes/src/ui/style.rs), [`EdgeStyleHook`](crates/egui-nodes/src/ui/style.rs)), etc.
- **Playground** (this repo’s root package **`egui-nodes-app`**): [`src/main.rs`](src/main.rs) — demo app; [`style_panel.rs`](src/style_panel.rs) edits [`NodesStyle`](crates/egui-nodes/src/ui/style.rs) / [`CanvasStyle`](crates/egui-nodes/src/ui/nodes_engine/canvas/mod.rs) for the canvas.

## Run

```sh
cargo run
```

[`default-members`](Cargo.toml) is `["."]`, so `cargo run` at the repo root builds and runs the **`egui-nodes-app`** binary ([`src/main.rs`](src/main.rs)).

## Dependencies

| Crate | Role | Depends on (summary) |
| --- | --- | --- |
| [`dag-lib`](crates/dag-lib) (import **`dag_lib`**) | Headless graph, evaluation, DAG helpers | Optional [`serde`](crates/dag-lib/Cargo.toml) |
| [`egui-nodes`](crates/egui-nodes) (import **`egui_nodes`**) | Editor + canvas + re-exported **`dag_lib`** | [`dag-lib`](crates/dag-lib), `egui`, etc. — [`crates/egui-nodes/Cargo.toml`](crates/egui-nodes/Cargo.toml) |
| **`egui-nodes-app`** (workspace root [`Cargo.toml`](Cargo.toml) `[package]`) | Demo only | [`egui-nodes`](crates/egui-nodes), `eframe`, `egui` |

Applications typically depend on **`egui-nodes`** (and use **`dag_lib`** through it or as a direct dep). Tools with no UI can depend on **`dag-lib`** alone.

## Architecture

**[`dag-lib`](crates/dag-lib)** — Directed dataflow: each [`Link`](crates/dag-lib/src/model.rs) is output-pin → input-pin. The graph is a **DAG** iff [`dependency_graph_is_acyclic`](crates/dag-lib/src/eval.rs) holds. The UI may hold temporary cycles; [`compute_topological_order`](crates/dag-lib/src/eval.rs) still yields an order for [`Executor`](crates/dag-lib/src/eval.rs). [`NodeId`](crates/dag-lib/src/ids.rs) / [`PinId`](crates/dag-lib/src/ids.rs) / [`LinkId`](crates/dag-lib/src/ids.rs) are opaque numeric ids (see crate docs).

**[`egui-nodes`](crates/egui-nodes)** — [`NodesEditor`](crates/egui-nodes/src/ui/editor/mod.rs) owns canonical [`Graph`](crates/dag-lib/src/model.rs) plus a slab [`NodeGraph`](crates/egui-nodes/src/ui/nodes_engine/mod.rs) for egui; [`NodeData`](crates/egui-nodes/src/ui/editor/mod.rs) maps nodes to payloads. Canvas (pan/zoom, wires, selection, [`NodeGraphViewer`](crates/egui-nodes/src/ui/nodes_engine/canvas/node_viewer.rs)) lives under **`egui_nodes::nodes_engine`**. [`layout_to_pos2`](crates/egui-nodes/src/ui/editor/mod.rs) / [`pos2_to_layout`](crates/egui-nodes/src/ui/editor/mod.rs) connect [`Layout2d`](crates/dag-lib/src/layout.rs) to egui space. After [`NodesView::show`](crates/egui-nodes/src/ui/view.rs), read edits via [`take_graph_changes`](crates/egui-nodes/src/ui/editor/mod.rs) on the editor.

**Root package** — Demo shell; it is not a library.

**Tests:** e.g. `graph_json_roundtrip` in [`model.rs`](crates/dag-lib/src/model.rs) — `cargo test -p dag-lib --features serde`.
