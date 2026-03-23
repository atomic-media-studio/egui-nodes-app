# egui-nodes-app

An `egui` application instancing two custom libraries:

- dag-lib: a headless Directed Acyclic Graph library
- egui-nodes: a library with a Node-Link environment for egui


![Rust CI](https://github.com/atomic-media-studio/egui-app/actions/workflows/rust-ci.yml/badge.svg)


## Run

```sh
# Development: Builds to 'target/debug/'
cargo run

# Distribution: Builds to 'target/release/'
cargo build --release
```

## Repository structure

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


[`default-members`](Cargo.toml) is `["."]`, so `cargo run` at the repo root builds and runs the **`egui-nodes-app`** binary ([`src/main.rs`](src/main.rs)).


## Dependencies

| Crate | Role | Depends on (summary) |
| --- | --- | --- |
| [`dag-lib`](crates/dag-lib) (import **`dag_lib`**) | Headless graph, evaluation, DAG helpers | Optional [`serde`](crates/dag-lib/Cargo.toml) |
| [`egui-nodes`](crates/egui-nodes) (import **`egui_nodes`**) | Editor + canvas + re-exported **`dag_lib`** | [`dag-lib`](crates/dag-lib), `egui`, etc. — [`crates/egui-nodes/Cargo.toml`](crates/egui-nodes/Cargo.toml) |
| **`egui-nodes-app`** (workspace root [`Cargo.toml`](Cargo.toml) `[package]`) | Demo only | [`egui-nodes`](crates/egui-nodes), `eframe`, `egui` |

Applications typically depend on **`egui-nodes`** (and use **`dag_lib`** through it or as a direct dep). Tools with no UI can depend on **`dag-lib`** alone.

## Architecture

**[`dag-lib`](crates/dag-lib)** - Directed dataflow: each [`Link`](crates/dag-lib/src/model.rs) is output-pin → input-pin. The graph is a **DAG** iff [`dependency_graph_is_acyclic`](crates/dag-lib/src/eval.rs) holds. The UI may hold temporary cycles; [`compute_topological_order`](crates/dag-lib/src/eval.rs) still yields an order for [`Executor`](crates/dag-lib/src/eval.rs). [`NodeId`](crates/dag-lib/src/ids.rs) / [`PinId`](crates/dag-lib/src/ids.rs) / [`LinkId`](crates/dag-lib/src/ids.rs) are opaque numeric ids (see crate docs).

**[`egui-nodes`](crates/egui-nodes)** — [`NodesEditor`](crates/egui-nodes/src/ui/editor/mod.rs) owns canonical [`Graph`](crates/dag-lib/src/model.rs) plus a slab [`NodeGraph`](crates/egui-nodes/src/ui/nodes_engine/mod.rs) for egui; [`NodeData`](crates/egui-nodes/src/ui/editor/mod.rs) maps nodes to payloads. Canvas (pan/zoom, wires, selection, [`NodeGraphViewer`](crates/egui-nodes/src/ui/nodes_engine/canvas/node_viewer.rs)) lives under **`egui_nodes::nodes_engine`**. [`layout_to_pos2`](crates/egui-nodes/src/ui/editor/mod.rs) / [`pos2_to_layout`](crates/egui-nodes/src/ui/editor/mod.rs) connect [`Layout2d`](crates/dag-lib/src/layout.rs) to egui space. After [`NodesView::show`](crates/egui-nodes/src/ui/view.rs), read edits via [`take_graph_changes`](crates/egui-nodes/src/ui/editor/mod.rs) on the editor.

**Root package** — Demo shell; it is not a library.

**Tests:** e.g. `graph_json_roundtrip` in [`model.rs`](crates/dag-lib/src/model.rs) — `cargo test -p dag-lib --features serde`.
