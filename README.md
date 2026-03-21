# egui-nodes-app

Workspace:

```text
Cargo.toml                      # workspace
crates/egui-snarl-fork/         # [lib] name = "egui-snarl" — forked node UI engine
crates/egui-nodes-app/          # one package: [lib] egui_nodes + [bin] egui-nodes-app
    src/lib.rs                  # reusable core: Graph, SnarlAdapter, NodesView, …
    src/main.rs                 # editor shell (eframe, menus, demo)
    src/style_panel.rs          # Snarl style UI (kept out of main.rs)
```

- **Library:** `egui_nodes` — same crate as the app; import as `egui_nodes::…` from `src/lib.rs` and submodules.
- **Binary:** [`egui-nodes-app`](crates/egui-nodes-app/src/main.rs) — `cargo run` builds the editor; Snarl style tuning lives in [`style_panel.rs`](crates/egui-nodes-app/src/style_panel.rs).

![Rust CI](https://github.com/atomic-media-studio/egui-app/actions/workflows/rust-ci.yml/badge.svg)

## Run

```sh
cargo run
```

(`default-members` points at `egui-nodes-app`.)

## Dependency rule

Application code uses the **`egui_nodes`** library (this package’s `[lib]`).  
The Snarl fork is a **path dependency of that library**; the binary does not add a separate Snarl dependency beyond the package `Cargo.toml`.
