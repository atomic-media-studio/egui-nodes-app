//! Stable identifiers for semantic nodes and edges.

/// Opaque id for a node in the semantic graph (not a Snarl slab index).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(transparent)]
pub struct SemanticNodeId(pub u64);

/// Opaque id for an edge in the semantic graph.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(transparent)]
pub struct SemanticEdgeId(pub u64);
