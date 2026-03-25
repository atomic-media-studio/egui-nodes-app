# Roadmap

## Ongoing

[ ] Add multiple tabs on the node viewer
[ ] `graph-lib` `eval.rs`: optional shared `Buffer` (e.g. `Arc<[f32]>`) on `Value` when you need block/sample data — not required until you touch DSP.
[ ] `graph-lib`: `NodeBehavior` (or similar) trait for headless semantics; migrate `DefaultNode` in `egui-nodes` to implement it.
[ ] `egui-nodes`: `NodeRegistry` driving the graph context menu (replace hardcoded spawn list + scattered `graph_menu` labels).
[ ] `graph-lib` `Executor`: dirty-flag / push-style evaluation alongside the existing full-graph pull pass.
[ ] Default node set: math layer — Add, Subtract, Multiply, Compare, Select (plus Max-style Max once ordering/`Value` fit).
[ ] Integration tests (graph round-trip, `NodeGraph` try-API); keep `cargo doc` / rustdoc warning-clean as APIs grow.
[ ] Optional `pub` re-exports for `CanvasState` / canvas internals if embedding apps need them without deep paths.
[ ] Demo: evaluation wiring from spawned nodes; palette grouping/icons once registry exists.

## Completed

[x] `graph-lib`: `PinType` (`pin_type.rs`), `Pin::ty` with serde default; `Graph::add_node_with_pin_types`; `Graph::connect` rejects incompatible types (`PinTypeMismatch`); wildcard `Any`; `NodesEditor::insert_node_with_pin_types`; `pin_types_for_default_node` + typed demo seed/spawns (`Float` → `Any` sink).
[x] `graph-lib` `eval.rs`: `Value` adds `Bang`, `Symbol(Arc<str>)`, `List(Vec<Value>)`; keeps `Bool` / `Int` / `Float`.
[x] Three-crate workspace: headless `graph-lib`, UI `egui-nodes`, demo app (`default-members` = app).
[x] `DefaultNode` / `DefaultNodeViewer` in `egui-nodes`: preset kinds; graph menu spawn requests queued and applied after `NodesView::show` (no nested `RefCell` borrows).
[x] Empty-canvas right-click graph menu: 160px width (`apply_graph_menu_width`); spawn Button / Int / String / Float / Sink; optional `print_graph_menu_*_clicked` hooks; viewport `Sense::click_and_drag`; menu on `select_resp.context_menu`.
[x] Snap-to-grid on node drag end (smooth drag); `CanvasStyle` snap step and toggle.
[x] Demo: side-panel snap checkbox; Node Inspector (three lines for selection); Backspace deletes selected nodes via `NodesEditor::remove_view_nodes`.
[x] Split `egui-nodes` canvas into `style`, `scene`, `draw`, `transform`, and `graph_menu` helpers; public canvas API stable at a high level.
[x] CI: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo doc -p egui-nodes --no-deps`, build, tests.
[x] Clippy/docs cleanups (`graph-lib` eval, style panel, background pattern, canvas borrows, intra-doc links, selection helpers).
[x] Wire sampling: resilient `sample_bezier`; removed dead commented code.
[x] Default editor/canvas styling (`NodesStyle::with_editor_canvas_defaults`) and side-panel live canvas controls.
[x] Canvas/grid, background pattern, selection UX (marquee without shift, consistent pin redraw).

