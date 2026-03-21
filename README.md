# egui-nodes-app

Template **nodes library** workspace: semantic graph core, Snarl adapter, egui view shell, and a small demo app over vendored [`egui-snarl`](crates/egui-snarl).

![Rust CI](https://github.com/atomic-media-studio/egui-app/actions/workflows/rust-ci.yml/badge.svg)

## Crates

| Crate | Role |
|--------|------|
| **`nodes-core`** | [`SemanticGraph<N, E>`](crates/nodes-core/src/graph.rs), ids, layout, [`save_graph`](crates/nodes-core/src/io.rs) / [`load_graph`](crates/nodes-core/src/io.rs) |
| **`nodes-snarl`** | [`SemanticSnarlBridge`](crates/nodes-snarl/src/bridge.rs): semantic ↔ Snarl id map, paired insert/connect/remove, [`sync_graph_from_snarl`](crates/nodes-snarl/src/bridge.rs), layout helpers |
| **`nodes-egui`** | [`NodesView`](crates/nodes-egui/src/view.rs), [`NodesShellViewer`](crates/nodes-egui/src/shell.rs), modes, styles |
| **`nodes-app`** | Minimal binary: semantic graph + bridge + Snarl + style panel |
| **`egui-snarl`** | Vendored node graph UI (fork) |

## Run

```sh
cargo run
cargo build --release
```

## Dependencies (app)

- `eframe` 
- `egui-phosphor`
- `nodes-core`, `nodes-snarl`, `nodes-egui` (path crates)
