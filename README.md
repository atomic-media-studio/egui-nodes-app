# egui-nodes-app

Workspace layout:

```text
Cargo.toml                 # workspace root + [package] playground binary
src/
  main.rs                  # editor demo — depends on egui-nodes only
  style_panel.rs           # Snarl style UI (optional; keeps main.rs small)
crates/
  core-graph/              # Graph + Pin topology, NonZeroU32 ids, eval (Executor / NodeEvaluator)
  egui-snarl-fork/         # forked Snarl (Rust crate: egui_snarl_fork)
  egui-nodes/              # Snarl adapter + NodesView + layout_bridge, io — uses core-graph + fork
    src/
      lib.rs
      snarl_adapter/       # SnarlAdapter + NodesShellViewer (viewer.rs)
      ui/                  # state, style, view (NodesView)
      layout_bridge.rs, io.rs
```

- `**core-graph**` (`[crates/core-graph](crates/core-graph)`) — portable graph model (`Node`/`Link`/`Pin`), `[compute_topological_order](crates/core-graph/src/eval.rs)`, `[Executor](crates/core-graph/src/eval.rs)` + `[NodeEvaluator](crates/core-graph/src/eval.rs)` + `[Value](crates/core-graph/src/eval.rs)`. Optional `serde`. No egui.
- `**egui-nodes**` (`[crates/egui-nodes](crates/egui-nodes)`) — import as `egui_nodes::…`. Depends on `**core-graph**` + `**egui-snarl-fork**`; re-exports `[core_graph](crates/egui-nodes/src/lib.rs)` and `[egui_snarl_fork](crates/egui-nodes/src/lib.rs)` for advanced use.
- **Playground:** `[src/main.rs](src/main.rs)` — `cargo run` from the repo root; Snarl tuning in `[style_panel.rs](src/style_panel.rs)`.

Rust CI

## Run

```sh
cargo run
```

`default-members` is `.` so the root `egui-nodes-app` binary is the default target.

## Dependency layers


| Crate            | Depends on                               |
| ---------------- | ---------------------------------------- |
| `core-graph`     | optional `serde` only                    |
| `egui-nodes`     | `core-graph`, `egui`, `egui-snarl-fork`  |
| `egui-nodes-app` | `egui-nodes` (and eframe for the window) |


GUI apps typically depend on `**egui-nodes**` only. A CLI can depend on `**core-graph**` alone.

## Architecture

- `**core-graph**` — Contract: topology, ids, `[Executor](crates/core-graph/src/eval.rs)` / `[NodeEvaluator](crates/core-graph/src/eval.rs)`. Benchmark and test here without egui.
- `**egui-nodes**` — Thin bridge: `[SnarlAdapter](crates/egui-nodes/src/snarl_adapter/mod.rs)` + `[NodesView](crates/egui-nodes/src/ui/view.rs)`. Use `[GraphChanges](crates/egui-nodes/src/snarl_adapter/mod.rs)` + `[take_graph_changes](crates/egui-nodes/src/snarl_adapter/mod.rs)` after each `[NodesView::show](crates/egui-nodes/src/ui/view.rs)` to re-run evaluation only when the graph actually changed.
- **Root binary** — Shell: window persistence (`persist_window`), toolbars, future project I/O — no graph semantics.

JSON round-trip coverage on [`Graph`] runs under `cargo test` when `core-graph` is built with the `serde` feature (including via workspace deps from `egui-nodes`).