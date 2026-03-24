# egui-nodes-app

Rust `egui` application instancing two custom libraries:

- graph-lib: a headless directed graph library
- egui-nodes: a node-link environment library for egui


![Rust CI](https://github.com/atomic-media-studio/egui-app/actions/workflows/rust-ci.yml/badge.svg)


## Run

```sh
# Development: Builds to 'target/debug/'
cargo run

# Distribution: Builds to 'target/release/'
cargo build --release

# Check documentation
cargo doc -p egui-nodes --no-deps --open
```

## Repository structure

```text
├── Cargo.toml                 # workspace root
├── src/                       # demo binary (default `cargo run`)
│   └── main.rs
└── crates/
    ├── graph-lib/             # Rust crate `graph_lib`
    └── egui-nodes/            # Rust crate `egui_nodes`
        └── src/
            ├── lib.rs         # public re-exports (see below)
            ├── io.rs          # load_graph / save_graph
            └── ui/            # editor, nodes_engine (NodeGraph + canvas), canvas_style_panel, …
```


[`default-members`](Cargo.toml) is `["."]`, so `cargo run` at the repo root builds and runs the **`egui-nodes-app`** binary ([`src/main.rs`](src/main.rs)).


## Dependencies

| Crate | Role | Depends on (summary) |
| --- | --- | --- |
| [`graph-lib`](crates/graph-lib) (import **`graph_lib`**) | Headless graph, evaluation, DAG helpers | Optional [`serde`](crates/graph-lib/Cargo.toml) |
| [`egui-nodes`](crates/egui-nodes) (import **`egui_nodes`**) | Editor + canvas + re-exported **`graph_lib`** | [`graph-lib`](crates/graph-lib), `egui`, etc. — [`crates/egui-nodes/Cargo.toml`](crates/egui-nodes/Cargo.toml) |
| **`egui-nodes-app`** (workspace root [`Cargo.toml`](Cargo.toml) `[package]`) | Demo only | [`egui-nodes`](crates/egui-nodes), `eframe`, `egui` |

Applications typically depend on **`egui-nodes`** (and use **`graph_lib`** through it or as a direct dep). Tools with no UI can depend on **`graph-lib`** alone.

