# egui-nodes-app

![Rust CI](https://github.com/atomic-media-studio/egui-app/actions/workflows/rust-ci.yml/badge.svg)

Repository structure:

```text
├── Cargo.toml                 # workspace root
├── src/                       # demo app (default `cargo run`)
│   ├── main.rs
│   └── style_panel.rs
└── crates/
    ├── dag-lib/src/           # headless Graph, DAG checks, Executor, …
    └── egui-nodes/src/
        ├── lib.rs, io.rs
        └── ui/                # editor/, nodes_engine/canvas/, state, style, view
```

- **`dag-lib`** (`[crates/dag-lib](crates/dag-lib)`) — portable directed graph (`Node`/`Link`/`Pin`), [`dependency_graph_is_acyclic`](crates/dag-lib/src/eval.rs), [`compute_topological_order`](crates/dag-lib/src/eval.rs), [`Executor`](crates/dag-lib/src/eval.rs) / [`NodeEvaluator`](crates/dag-lib/src/eval.rs) / [`Value`](crates/dag-lib/src/eval.rs). Optional `serde`. No egui.
- **`egui-nodes`** (`[crates/egui-nodes](crates/egui-nodes)`) — import as `egui_nodes::…`. Depends on **`dag-lib`** and egui; re-exports [`dag_lib`](crates/egui-nodes/src/lib.rs) and [`nodes_engine`](crates/egui-nodes/src/lib.rs) (interactive [`NodeGraph`](crates/egui-nodes/src/ui/nodes_engine/mod.rs) + canvas).
- **Playground:** [`src/main.rs`](src/main.rs) — `cargo run` from the repo root; canvas tuning in [`style_panel.rs`](src/style_panel.rs).

## Run

```sh
cargo run
```

The workspace’s [`default-members`](Cargo.toml) is `["."]`, so `cargo run` from the repo root runs the **`egui-nodes-app`** binary ([`src/main.rs`](src/main.rs)).

## Dependencies

| Crate | Role | Depends on (summary) |
| --- | --- | --- |
| [`dag-lib`](crates/dag-lib) | Headless graph + [`Executor`](crates/dag-lib/src/eval.rs) | Optional [`serde`](crates/dag-lib/Cargo.toml) only |
| [`egui-nodes`](crates/egui-nodes) | UI: [`NodeGraph`](crates/egui-nodes/src/ui/nodes_engine/mod.rs), canvas, [`NodesEditor`](crates/egui-nodes/src/ui/editor/mod.rs) | [`dag-lib`](crates/dag-lib), `egui`, and other deps in [`crates/egui-nodes/Cargo.toml`](crates/egui-nodes/Cargo.toml) |
| **`egui-nodes-app`** (root [`Cargo.toml`](Cargo.toml) `[package]`) | Demo window + style panel | [`egui-nodes`](crates/egui-nodes), `eframe`, `egui` |

Apps usually depend on **`egui-nodes`** only; headless tools can use **`dag-lib`** alone.

## Architecture

**[`dag-lib`](crates/dag-lib)** — **Directed** dataflow graph: each [`Link`](crates/dag-lib/src/model.rs) goes from an **output** pin to an **input** pin. **DAG** (acyclic) in the usual graph-theory sense iff [`dependency_graph_is_acyclic`](crates/dag-lib/src/eval.rs) is true (Kahn completes). The editor may temporarily store cycles; [`compute_topological_order`](crates/dag-lib/src/eval.rs) then appends unsorted nodes so [`Executor`](crates/dag-lib/src/eval.rs) still runs. **Ids:** [`NodeId`](crates/dag-lib/src/ids.rs) / [`PinId`](crates/dag-lib/src/ids.rs) / [`LinkId`](crates/dag-lib/src/ids.rs) are global opaque `u32` handles — not string keys (see crate docs).

**[`egui-nodes`](crates/egui-nodes)** — [`NodesEditor`](crates/egui-nodes/src/ui/editor/mod.rs) syncs canonical [`Graph`](crates/dag-lib/src/model.rs) with the slab [`NodeGraph`](crates/egui-nodes/src/ui/nodes_engine/mod.rs) (canvas indices ↔ [`PinId`](crates/dag-lib/src/ids.rs)); wire drawing and hit-tests stay in the UI crate. [`layout_to_pos2`](crates/egui-nodes/src/ui/editor/mod.rs) bridges [`Layout2d`](crates/dag-lib/src/layout.rs) and egui. After [`NodesView::show`](crates/egui-nodes/src/ui/view.rs), use [`take_graph_changes`](crates/egui-nodes/src/ui/editor/mod.rs).

**Root** — Demo shell only; no graph semantics beyond the library.

**Tests:** `graph_json_roundtrip` in [`model.rs`](crates/dag-lib/src/model.rs) — `cargo test -p dag-lib --features serde`.
