# Roadmap

## Ongoing

[ ] Add integration tests (e.g. graph round-trip, `NodeGraph` try-API) and optional `cargo doc` with `-D warnings` once rustdoc is fully clean.
[ ] Consider `pub` re-exports for `CanvasState` / canvas internals if embedding apps need typed access without deep paths.
[ ] Demo app: extend graph menu or node palette (e.g. grouping, icons, spawn without duplicating type labels) and optional graph evaluation wiring from spawned nodes.

## Completed

[x] **Demo / canvas UX:** Empty-canvas right-click **graph menu** (fixed **160px** width): spawn Button / Int / String / Float / Sink; dedicated `print_graph_menu_*_clicked` stdout hooks; “last spawn” line in the central panel. Library: viewport `Sense::click_and_drag`, graph menu on `select_resp.context_menu` so right-clicks open the menu on empty canvas.
[x] Split `egui-nodes` canvas into `style`, `scene`, `draw`, and `transform` modules with module-level docs; public `nodes_canvas` API unchanged.
[x] CI: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo doc -p egui-nodes --no-deps`, build, and tests.
[x] Clippy-driven cleanups (`graph-lib` eval, style panel `unwrap_or_default`, background pattern `too_many_arguments`, canvas needless borrows).
[x] Wire sampling: removed dead commented code; `sample_bezier` handles empty and high-degree control polygons without `unimplemented!`.
[x] Docs: fixed broken/redundant intra-doc links in crate root, editor, canvas, `state`, `style`, `background_pattern`, `node_viewer`.
[x] Removed unused `deselect_one_node` / `deselect_many_nodes` from selection state; demo top bar no longer shows inert icon buttons.

[x] Three-crate workspace: headless `graph-lib`, UI `egui-nodes`, demo binary (`default-members` = app).
[x] Default editor/canvas styling (`NodesStyle::with_editor_canvas_defaults`) and side-panel live controls (`canvas_style_controls_ui`).
[x] Canvas/grid and background-pattern refactor, optimizations, and selection UX (marquee without shift, consistent pin redraw).

