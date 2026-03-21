//! **Snarl adapter** — map [`nodes_core::SemanticNodeId`] ↔ [`egui_snarl::NodeId`], keep
//! [`nodes_core::SemanticGraph`] and [`egui_snarl::Snarl`] in sync, convert [`nodes_core::Layout2d`]
//! to [`egui::Pos2`].
//!
//! Application code should prefer [`SemanticSnarlBridge`] for inserts/removes/connects that must
//! appear in both the semantic model and the editor. After each frame of interaction, call
//! [`SemanticSnarlBridge::sync_graph_from_snarl`] so the semantic graph stays the serialized
//! source of truth.

pub mod bridge;
pub mod layout_map;

pub use bridge::{BridgeError, SemanticSnarlBridge};
pub use layout_map::{layout_to_pos2, pos2_to_layout};

pub use egui_snarl;

pub use nodes_core::{
    GraphError, Layout2d, SemanticEdge, SemanticEdgeId, SemanticGraph, SemanticNode, SemanticNodeId,
};
