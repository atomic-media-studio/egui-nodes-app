//! **Semantic graph** — domain nodes, edges, and 2D layout without egui types.
//!
//! Use this crate for evaluation, transforms, and serialization of *meaning*. Pair it with
//! `egui_snarl::Snarl` (via `nodes-snarl`) for editor layout and interaction.
//!
//! ## Split of responsibilities
//!
//! - **`SemanticGraph<N, E>`** — stable [`SemanticNodeId`](id::SemanticNodeId), topology, payloads.
//! - **Snarl** (`egui-snarl` crate) — UI slab ids, positions, wires, and interaction.
//!
//! Serialize [`SemanticGraph`] as your source of truth; you can additionally persist `Snarl` with the
//! `serde` feature on `egui-snarl` for a full editor snapshot.

pub mod error;
pub mod graph;
pub mod id;
pub mod layout;

pub use error::GraphError;
pub use graph::{SemanticEdge, SemanticGraph, SemanticNode};
pub use id::{SemanticEdgeId, SemanticNodeId};
pub use layout::Layout2d;

mod io;

pub use io::{load_graph, save_graph};
