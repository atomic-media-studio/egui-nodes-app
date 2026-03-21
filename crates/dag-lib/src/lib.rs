//! **dag-lib** — directed graph model for **dataflow**: nodes, typed pins, and links from outputs to inputs.
//!
//! # Directed acyclic graph (DAG) semantics
//!
//! In the usual sense for **compilers and dataflow** ([DAG](https://en.wikipedia.org/wiki/Directed_acyclic_graph)):
//! a **directed** edge goes from a producer to a consumer; **acyclicity** means no directed cycle, so a
//! [topological ordering](https://en.wikipedia.org/wiki/Topological_sorting) exists. This crate’s
//! [`Graph`] stores that structure: each [`Link`] connects an **output** [`PinId`] to an **input** [`PinId`],
//! inducing a directed graph on **nodes**. [`dependency_graph_is_acyclic`] is true exactly when that
//! node graph is a DAG (Kahn’s algorithm completes without leftover nodes).
//!
//! The [`Graph`] type does **not** reject cyclic edits at insert time (useful for editors that let users
//! draw wires before fixing cycles). [`compute_topological_order`] therefore appends any unsorted nodes
//! in a stable order so [`Executor`] still terminates; for **semantically correct** single-pass
//! dataflow evaluation, keep the graph acyclic or fix cycles before relying on pin values.
//!
//! # Identifiers
//!
//! [`NodeId`], [`PinId`], and [`LinkId`] are **opaque, globally unique** handles (`NonZeroU32`) allocated
//! by [`Graph`]. They are stable for the lifetime of the graph and are **not** slab indices or egui
//! widget ids. Optional human-readable names belong in your node payload `N` or labels — not in these
//! ids. String keys are heavier and better suited for file interchange or external tool integration;
//! the numeric ids stay compact for hot paths and hashing.
//!
//! # Layout
//!
//! [`Layout2d`] stores editor positions for nodes without depending on any UI crate.

pub mod error;
pub mod eval;
pub mod ids;
pub mod layout;
pub mod model;
pub mod serde;

pub use error::GraphError;
pub use eval::{
    EvalContext, Executor, NodeEvaluator, Value, compute_topological_order,
    dependency_graph_is_acyclic, gather_inputs_for_node,
};
pub use ids::{LinkId, NodeId, PinId};
pub use layout::Layout2d;
pub use model::{Graph, Link, Node, Pin, PinKind};
