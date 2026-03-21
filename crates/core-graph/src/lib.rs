//! **core-graph** — headless graph model and (optional) evaluation for this workspace.
//!
//! Provides stable ids, [`Graph`] topology, [`Pin`] connectivity, [`Layout2d`], and with the
//! `serde` feature, portable snapshots. Evaluation and traversal helpers live in [`eval`].
//!
//! **Design**: keep benchmarks, fuzzing, and hot-path graph logic here; UI crates depend on this
//! crate and map their widgets to [`NodeId`] / [`LinkId`] without duplicating topology rules.

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
