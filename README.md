# egui-nodes-app

![Rust CI](https://github.com/atomic-media-studio/egui-app/actions/workflows/rust-ci.yml/badge.svg)


Repository structure:

```text
├── Cargo.toml                 # workspace root
├── src/                       # demo app (default `cargo run`)
│   ├── main.rs
│   └── style_panel.rs
└── crates/
    ├── core-graph/src/        # headless Graph, Executor, …
    └── egui-nodes/src/
        ├── lib.rs, io.rs, layout_bridge.rs
        └── ui/                # editor/, nodes_engine/canvas/, state, style, view
```

- `**core-graph**` (`[crates/core-graph](crates/core-graph)`) — portable graph model (`Node`/`Link`/`Pin`), `[compute_topological_order](crates/core-graph/src/eval.rs)`, `[Executor](crates/core-graph/src/eval.rs)` + `[NodeEvaluator](crates/core-graph/src/eval.rs)` + `[Value](crates/core-graph/src/eval.rs)`. Optional `serde`. No egui.
- `**egui-nodes**` (`[crates/egui-nodes](crates/egui-nodes)`) — import as `egui_nodes::…`. Depends on `**core-graph**` and egui; re-exports `[core_graph](crates/egui-nodes/src/lib.rs)` and `[nodes_engine](crates/egui-nodes/src/lib.rs)` (interactive [`NodeGraph`](crates/egui-nodes/src/ui/nodes_engine/mod.rs) + canvas).
- **Playground:** `[src/main.rs](src/main.rs)` — `cargo run` from the repo root; canvas tuning in `[style_panel.rs](src/style_panel.rs)`.


## Run

```sh
cargo run
```

The workspace’s [`default-members`](Cargo.toml) is `["."]`, so `cargo run` from the repo root runs the **`egui-nodes-app`** binary (`src/main.rs`).

## Dependencies

| Crate | Role | Depends on (summary) |
| --- | --- | --- |
| [`core-graph`](crates/core-graph) | Headless graph + [`Executor`](crates/core-graph/src/eval.rs) | Optional [`serde`](crates/core-graph/Cargo.toml) only |
| [`egui-nodes`](crates/egui-nodes) | UI: [`NodeGraph`](crates/egui-nodes/src/ui/nodes_engine/mod.rs), canvas, [`NodesEditor`](crates/egui-nodes/src/ui/editor/mod.rs) | [`core-graph`](crates/core-graph), `egui`, and other deps in [`crates/egui-nodes/Cargo.toml`](crates/egui-nodes/Cargo.toml) |
| **`egui-nodes-app`** (root [`Cargo.toml`](Cargo.toml) `[package]`) | Demo window + style panel | [`egui-nodes`](crates/egui-nodes), `eframe`, `egui` |

Apps usually depend on **`egui-nodes`** only; headless tools can use **`core-graph`** alone.

## Architecture

- **[`core-graph`](crates/core-graph)** — Topology, [`NodeId`](crates/core-graph/src/ids.rs) / [`PinId`](crates/core-graph/src/ids.rs), [`Executor`](crates/core-graph/src/eval.rs) and [`NodeEvaluator`](crates/core-graph/src/eval.rs). No egui; good place for unit tests and benchmarks.
- **[`egui-nodes`](crates/egui-nodes)** — [`NodesEditor`](crates/egui-nodes/src/ui/editor/mod.rs) keeps [`Graph`](crates/core-graph/src/model.rs) and the slab [`NodeGraph`](crates/egui-nodes/src/ui/nodes_engine/mod.rs) in sync; [`NodesView`](crates/egui-nodes/src/ui/view.rs) drives the canvas. After each [`NodesView::show`](crates/egui-nodes/src/ui/view.rs), call [`take_graph_changes`](crates/egui-nodes/src/ui/editor/mod.rs) and inspect [`GraphChanges`](crates/egui-nodes/src/ui/editor/mod.rs) to refresh an executor only when topology or payloads changed.
- **Root crate** — Window + demo UI only (`persist_window`, panels); no graph logic beyond calling the library.

JSON round-trip for [`Graph`](crates/core-graph/src/model.rs) is tested in `graph_json_roundtrip` (same file) when `core-graph` is built with `--features serde` — e.g. `cargo test -p core-graph --features serde` (plain `cargo test -p core-graph` skips that test).
