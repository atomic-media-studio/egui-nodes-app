# egui-nodes-app

Workspace layout:

| Path | Crate | Purpose |
|------|--------|---------|
| `crates/egui-snarl-fork` | **`egui-snarl`** | Fork of the node-graph UI engine (package name unchanged → `use egui_snarl::…`). |
| `crates/egui-nodes` | **`egui-nodes`** | Headless [`Graph<N, E>`](crates/egui-nodes/src/graph/mod.rs), [`SnarlAdapter`](crates/egui-nodes/src/adapter.rs), [`NodesView`](crates/egui-nodes/src/view.rs) — **apps should depend on this**, not on the fork directly. |
| `crates/nodes-app` | **`nodes-app`** | Demo / template binary. |

![Rust CI](https://github.com/atomic-media-studio/egui-app/actions/workflows/rust-ci.yml/badge.svg)

## Run

```sh
cargo run
```

## Dependency rule

- **Application code**: `egui-nodes` (+ `eframe`, etc.).
- **Low-level Snarl**: only if you need it — `egui_nodes::egui_snarl` is re-exported for power users.

## Foundational graph (headless)

[`egui_nodes::Graph<N, E>`](crates/egui-nodes/src/graph/mod.rs) uses [`NodeId`](crates/egui-nodes/src/graph/id.rs), [`PinId`](crates/egui-nodes/src/graph/id.rs), [`LinkId`](crates/egui-nodes/src/graph/id.rs) and does **not** depend on egui. Serialize with [`egui_nodes::save_graph`](crates/egui-nodes/src/io.rs) / [`load_graph`](crates/egui-nodes/src/io.rs) when `serde` is enabled on your payload types.
