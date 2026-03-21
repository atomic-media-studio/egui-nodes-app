//! **core-graph** — the headless **contract** for this workspace: stable ids, [`Graph`] topology,
//! optional [`eval`] dataflow execution, and (with the `serde` feature) portable snapshots.
//!
//! Keep all real-time semantics, benchmarks, and fuzz targets here so **egui never sits on the hot
//! path** for graph logic. GUI crates should only adapt and visualize.

pub mod error;
pub mod eval;
pub mod ids;
pub mod layout;
pub mod model;
pub mod serde;

pub use error::GraphError;
pub use eval::{
    EvalContext, Executor, NodeEvaluator, Value, compute_topological_order, gather_inputs_for_node,
};
pub use ids::{LinkId, NodeId, PinId};
pub use layout::Layout2d;
pub use model::{Graph, Link, Node, Pin, PinKind};
