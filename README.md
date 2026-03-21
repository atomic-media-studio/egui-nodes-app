# egui-nodes-app

Workspace layout:

```text
Cargo.toml                 # workspace root + [package] playground binary
src/
  main.rs                  # editor demo — depends on egui-nodes only
  style_panel.rs           # Snarl style UI (optional; keeps main.rs small)
crates/
  core-graph/              # headless Graph, Node, Link, ids — no egui (CLI-ready)
  egui-snarl-fork/         # forked Snarl (Rust crate: egui_snarl_fork)
  egui-nodes/              # Snarl adapter + NodesView + layout_bridge, io — uses core-graph + fork
    src/
      lib.rs
      snarl_adapter/       # SnarlAdapter + NodesShellViewer (viewer.rs)
      ui/                  # state, style, view (NodesView)
      layout_bridge.rs, io.rs
```

- **`core-graph`** ([`crates/core-graph`](crates/core-graph)) — portable graph model and logic only (`serde` optional). Use this from a future CLI or any non-egui frontend.
- **`egui-nodes`** ([`crates/egui-nodes`](crates/egui-nodes)) — import as `egui_nodes::…`. Depends on **`core-graph`** + **`egui-snarl-fork`**; re-exports [`core_graph`](crates/egui-nodes/src/lib.rs) and [`egui_snarl_fork`](crates/egui-nodes/src/lib.rs) for advanced use.
- **Playground:** [`src/main.rs`](src/main.rs) — `cargo run` from the repo root; Snarl tuning in [`style_panel.rs`](src/style_panel.rs).

![Rust CI](https://github.com/atomic-media-studio/egui-app/actions/workflows/rust-ci.yml/badge.svg)

## Run

```sh
cargo run
```

`default-members` is `.` so the root `egui-nodes-app` binary is the default target.

## Dependency layers

| Crate            | Depends on                         |
|-----------------|-------------------------------------|
| `core-graph`    | optional `serde` only               |
| `egui-nodes`    | `core-graph`, `egui`, `egui-snarl-fork` |
| `egui-nodes-app`| `egui-nodes` (and eframe for the window) |

GUI apps typically depend on **`egui-nodes`** only. A CLI can depend on **`core-graph`** alone.
