use core::fmt;

use crate::id::{SemanticEdgeId, SemanticNodeId};

/// Errors from graph operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GraphError {
    UnknownNode(SemanticNodeId),
    UnknownEdge(SemanticEdgeId),
    DuplicateEdge {
        from: SemanticNodeId,
        out_port: usize,
        to: SemanticNodeId,
        in_port: usize,
    },
    SelfConnection {
        node: SemanticNodeId,
    },
}

impl fmt::Display for GraphError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownNode(id) => write!(f, "unknown semantic node {:?}", id.0),
            Self::UnknownEdge(id) => write!(f, "unknown semantic edge {:?}", id.0),
            Self::DuplicateEdge {
                from,
                out_port,
                to,
                in_port,
            } => write!(
                f,
                "duplicate edge {:?}:{} -> {:?}:{}",
                from.0, out_port, to.0, in_port
            ),
            Self::SelfConnection { node } => {
                write!(f, "cannot connect node {:?} to itself", node.0)
            }
        }
    }
}

impl std::error::Error for GraphError {}
